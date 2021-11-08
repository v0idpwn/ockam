defmodule Ockam.Session.Routing.Pluggable do
  @moduledoc """
  Routing session

  Initiator sends a first handshake message to the init_route on start
  and waits for a response from the responder.

  Responder receives the handshake, starts the data worker and sends a response

  After receiving the response, initiator starts the data worker.

  Utilizing pluggable handshake logic using `Ockam.Session.Handshake` behaviour
  """

  @doc """
  Shared function for data stage of the session

  State MUST have :data_state and :worker_mod keys
  """
  @spec handle_data_message(any(), %{:data_state => any(), :worker_mod => atom(), any() => any()}) ::
          {:ok, %{data_state: any()}} | {:error, any()} | {:stop, any(), %{data_state: any()}}
  def handle_data_message(message, state) do
    data_state = Map.fetch!(state, :data_state)
    worker_mod = Map.fetch!(state, :worker_mod)

    case worker_mod.handle_message(message, data_state) do
      {:ok, new_data_state} ->
        {:ok, Map.put(state, :data_state, new_data_state)}

      {:error, error} ->
        {:error, {:data_error, error}}

      {:stop, reason, new_data_state} ->
        {:stop, reason, Map.put(state, :data_state, new_data_state)}
    end
  end
end

## First option: non-buffering session, messages are dropped if the session is not ready yet
defmodule Ockam.Session.Routing.Pluggable.Initiator do
  @moduledoc """
  Simple routing session initiator

  Upon starting, uses Handshake.init to generate a handshake message
  and send it to init_route.
  Initial stage is :handshake, in this stage waits for a handshake response

  After receiving a handshake response, runs Handshake.handle_initiator
  and starts the data worker on the same process and moves to the :data stage

  Data worker is started with `worker_options` merged with
  the options from handle_initiator

  In the :data stage processes all messages with the data worker module

  Options:

  `init_route` - route to responder (or spawner)

  `worker_mod` - data worker module
  `worker_options` - data worker options

  `handshake` - handshake module (defaults to `Ockam.Session.Handshake.Default`)
  `handshake_options` - options for handshake module
  """
  use Ockam.AsymmetricWorker

  alias Ockam.Message
  alias Ockam.Router

  alias Ockam.Session.Routing.Pluggable, as: RoutingSession

  require Logger

  @dialyzer {:nowarn_function, handle_inner_message: 2, handle_outer_message: 2}

  def get_stage(worker) do
    Ockam.Worker.call(worker, :get_stage)
  end

  def wait_for_session(worker, interval \\ 100, timeout \\ 5000)

  def wait_for_session(_worker, _interval, expire) when expire < 0 do
    {:error, :timeout}
  end

  def wait_for_session(worker, interval, timeout) do
    case get_stage(worker) do
      :data ->
        :ok

      :handshake ->
        :timer.sleep(interval)
        wait_for_session(worker, interval, timeout - interval)
    end
  end

  def create_and_wait(options, interval \\ 50, timeout \\ 5000) do
    with {:ok, address} <- create(options),
         :ok <- wait_for_session(address, interval, timeout) do
      {:ok, address}
    end
  end

  @impl true
  def address_prefix(_options), do: "S_I_"

  @impl true
  def inner_setup(options, state) do
    init_route = Keyword.fetch!(options, :init_route)

    base_state = state
    ## rename to data_mod
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    handshake = Keyword.get(options, :handshake, Ockam.Session.Handshake.Default)
    handshake_options = Keyword.get(options, :handshake_options, [])

    state =
      Map.merge(state, %{
        init_route: init_route,
        worker_mod: worker_mod,
        worker_options: worker_options,
        base_state: base_state,
        handshake: handshake,
        handshake_options: handshake_options
      })

    state = send_handshake(handshake_options, state)

    {:ok, Map.put(state, :stage, :handshake)}
  end

  def send_handshake(handshake_options, state) do
    handshake = Map.fetch!(state, :handshake)
    {:ok, handshake_msg, state} = handshake.init(handshake_options, state)
    Logger.info("handshake_msg: #{inspect(handshake_msg)}")
    Ockam.Router.route(handshake_msg)
    state
  end

  @impl true
  def handle_call(:get_stage, _from, state) do
    {:reply, Map.get(state, :stage), state}
  end

  @impl true
  def handle_message(message, %{stage: :handshake} = state) do
    case message_type(message, state) do
      :inner ->
        handle_handshake_message(message, state)

      _other ->
        Logger.info("Ignoring non-inner message in handshake stage: #{inspect(message)}")
        {:ok, state}
    end
  end

  def handle_message(message, %{stage: :data, data_state: _, worker_mod: _} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  def handle_handshake_message(message, state) do
    handshake = Map.fetch!(state, :handshake)
    handshake_options = Map.fetch!(state, :handshake_options)

    case handshake.handle_initiator(handshake_options, message, state) do
      {:ok, options, state} ->
        switch_to_data_stage(options, state)

      {:error, err} ->
        ## TODO: error handling in Ockam.Worker
        {:error, err}
    end
  end

  def switch_to_data_stage(handshake_options, state) do
    base_state = Map.get(state, :base_state)
    worker_mod = Map.fetch!(state, :worker_mod)
    worker_options = Map.fetch!(state, :worker_options)

    options = Keyword.merge(worker_options, handshake_options)

    case worker_mod.setup(options, base_state) do
      {:ok, data_state} ->
        {:ok, Map.merge(state, %{data_state: data_state, stage: :data})}

      {:error, err} ->
        {:stop, {:cannot_start_data_worker, {:error, err}, options, base_state}, state}
    end
  end
end

## Single handshake responder
defmodule Ockam.Session.Routing.Pluggable.Responder do
  @moduledoc """
  Routing session responder

  If :init_message is present in the options - processes the message,
  otherwise waits for it in :handshake stage

  On processing the handshake calls `handshake.handle_responder/1`, which
  generates handshake response message and options

  Starts the data worker with worker_options merged with
  the options from `handshake.handle_responder/1`

  If worker started successfully, sends the handshake response
  and moves to the :data stage

  All messages in :data stage are processed with the data worker module

  Options:

  `worker_mod` - data worker module
  `worker_options` - data worker options, defaults to []

  `handshake` - handshake module (defaults to `Ockam.Session.Handshake.Default`)
  `handshake_options` - options for handshake module, defaults to []

  `init_message` - optional init message
  """
  use Ockam.AsymmetricWorker

  alias Ockam.Message
  alias Ockam.Session.Routing.Pluggable, as: RoutingSession

  require Logger

  @dialyzer {:nowarn_function, handle_inner_message: 2, handle_outer_message: 2}

  @impl true
  def address_prefix(_options), do: "S_R_"

  @impl true
  def inner_setup(options, state) do
    base_state = state
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    handshake = Keyword.get(options, :handshake, Ockam.Session.Handshake.Default)
    handshake_options = Keyword.get(options, :handshake_options, [])

    state =
      Map.merge(state, %{
        worker_mod: worker_mod,
        worker_options: worker_options,
        base_state: base_state,
        stage: :handshake,
        handshake: handshake,
        handshake_options: handshake_options
      })

    case Keyword.get(options, :init_message) do
      nil ->
        ## Stay in the handshake stage, wait for init message
        {:ok, state}

      %{payload: _} = message ->
        handle_handshake_message(message, state)
    end
  end

  @impl true
  def handle_message(message, %{stage: :handshake} = state) do
    case message_type(message, state) do
      :inner ->
        handle_handshake_message(message, state)

      _other ->
        ## TODO: buffering option?
        Logger.debug("Ignoring non-inner message #{inspect(message)} in handshake stage")
        {:ok, state}
    end
  end

  def handle_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  def handle_handshake_message(message, state) do
    handshake = Map.fetch!(state, :handshake)
    handshake_options = Map.fetch!(state, :handshake_options)

    case handshake.handle_responder(handshake_options, message, state) do
      {:ok, response, options, state} ->
        switch_to_data_stage(response, options, state)

      {:error, err} ->
        {:error, err}
    end
  end

  defp switch_to_data_stage(response, handshake_options, state) do
    worker_mod = Map.fetch!(state, :worker_mod)
    worker_options = Map.fetch!(state, :worker_options)
    base_state = Map.fetch!(state, :base_state)

    options = Keyword.merge(worker_options, handshake_options)

    case worker_mod.setup(options, base_state) do
      {:ok, data_state} ->
        send_handshake_response(response)
        ## TODO: use data_state instead of state futher on?
        {:ok, Map.merge(state, %{data_state: data_state, stage: :data})}

      {:error, err} ->
        Logger.error(
          "Error starting responder data module: #{worker_mod}, reason: #{inspect(err)}"
        )

        ## TODO: should we send handshake error?
        {:error, err}
    end
  end

  def send_handshake_response(response) do
    Ockam.Router.route(response)
  end
end
