defmodule Ockam.Session.Routing do
  @moduledoc """
  Simple routing session logic.
  Initiator sends an empty message to the spawner on start
  and waits for a response from the responder.
  """

  @doc """
  Shared function for data stage of the session

  State MUST have :data_state and :worker_mod keys
  """
  @spec handle_data_message(Ockam.Message.t(), %{data_state: any(), worker_mod: atom()}) :: %{
          data_state: any()
        }
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

## TODO asymmetric worker for session
## The main issue is that if internal worker is asymmetric
## it might override the internal address

## First option: non-buffering session, messages are dropped if the session is not ready yet
defmodule Ockam.Session.Routing.Initiator do
  @moduledoc """
  Simple routing session initiator

  Upon starting sends a message with empty payload to the responder and
  waits for a response in the :handshake stage

  After receiving a handshake response, starts the data worker on the same process and
  moves to the :data stage

  Data worker initial state is the same as the session worker initial state
  Data worker is started with `worker_options` merged with
  handshake response message return route as `:route`
  and handshake payload as `:handshake_data`

  In the :data stage processes all messages with the data worker module

  Options:

  `worker_mod` - data worker module
  `worker_options` - data worker options
  `init_route` - route to responder (or spawner)
  """
  use Ockam.AsymmetricWorker

  alias Ockam.Message
  alias Ockam.Router

  alias Ockam.Session.Routing, as: RoutingSession

  require Logger

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

  @impl true
  def inner_setup(options, state) do
    init_route = Keyword.fetch!(options, :init_route)

    base_state = state
    ## rename to data_mod
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    state =
      Map.merge(state, %{
        init_route: init_route,
        worker_mod: worker_mod,
        worker_options: worker_options,
        base_state: base_state
      })

    state = send_handshake(state)
    {:ok, Map.put(state, :stage, :handshake)}
  end

  @impl true
  def handle_call(:get_stage, _from, state) do
    {:reply, Map.get(state, :stage), state}
  end

  @impl true
  def handle_inner_message(message, %{stage: :handshake} = state) do
    case is_handshake?(message) do
      true ->
        handle_handshake_message(message, state)

      false ->
        Logger.debug("Ignoring message #{inspect(message)} in handshake stage")
        {:ok, state}
    end
  end

  def handle_inner_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  @impl true
  def handle_outer_message(message, %{stage: :handshake} = state) do
    Logger.debug("Ignoring outer message #{inspect(message)} in handshake stage")
    {:ok, state}
  end

  def handle_outer_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  @impl true
  def handle_other_message(message, %{stage: :handshake} = state) do
    Logger.debug("Ignoring other message #{inspect(message)} in handshake stage")
    {:ok, state}
  end

  def handle_other_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  @impl true
  def handle_non_message(message, %{stage: :handshake} = state) do
    Logger.debug("Ignoring non-ockam message #{inspect(message)} in handshake stage")
    {:ok, state}
  end

  def handle_non_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  def is_handshake?(message) do
    ## TODO: validate if message is a handshake
    Message.payload(message) == ""
  end

  def handle_handshake_message(message, state) do
    data_route = Message.return_route(message)
    handshake_data = Message.payload(message)

    switch_to_data_stage(data_route, handshake_data, state)
  end

  def switch_to_data_stage(data_route, handshake_data, state) do
    base_state = Map.get(state, :base_state)
    worker_mod = Map.fetch!(state, :worker_mod)
    worker_options = Map.fetch!(state, :worker_options)

    options = Keyword.merge(worker_options, route: data_route, handshake_data: handshake_data)

    case worker_mod.setup(options, base_state) do
      {:ok, data_state} ->
        {:ok, Map.merge(state, %{data_state: data_state, stage: :data})}

      {:error, err} ->
        {:stop, {:cannot_start_data_worker, {:error, err}, options, base_state}, state}
    end
  end

  def send_handshake(state) do
    init_route = Map.fetch!(state, :init_route)

    Router.route(%{
      onward_route: init_route,
      return_route: [state.inner_address],
      payload: ""
    })

    state
  end
end

## Single handshake responder
defmodule Ockam.Session.Routing.Responder do
  @moduledoc """
  Simple routing session responder

  Started with :initiator_route and :handshake_data
  On start initializes the data worker and if successful sends a handshake response

  Data worker is started with `:initiator_route` as `:route`
  and `:handshake_data` as `:handshake_data` options

  All messages are processed with the data worker module
  """
  use Ockam.AsymmetricWorker

  alias Ockam.Message
  alias Ockam.Session.Routing, as: RoutingSession

  require Logger

  @impl true
  def inner_setup(options, state) do
    base_state = state
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.fetch!(options, :worker_options)

    state =
      Map.merge(state, %{
        worker_mod: worker_mod,
        worker_options: worker_options,
        base_state: base_state,
        stage: :handshake
      })

    case Keyword.get(options, :init_message) do
      nil ->
        ## Stay in the handshake stage, wait for init message
        {:ok, state}

      %{payload: _} = message ->
        handle_init_message(message, state)
    end
  end

  @impl true
  def handle_message(message, %{stage: :handshake} = state) do
    case message_type(message, state) do
      :inner ->
        handle_init_message(message, state)

      _ ->
        Logger.debug("Ignoring message #{inspect(message)} in handshake stage")
        {:ok, state}
    end
  end

  def handle_message(message, %{stage: :data} = state) do
    RoutingSession.handle_data_message(message, state)
  end

  # @impl true
  # def handle_other_message(message, %{stage: :handshake} = state) do
  #   Logger.debug("Ignoring other message #{inspect(message)} in handshake stage")
  #   {:ok, state}
  # end
  # def handle_other_message(message, %{stage: :data} = state) do
  #   RoutingSession.handle_data_message(message, state)
  # end

  # @impl true
  # def handle_non_message(message, %{stage: :handshake} = state) do
  #   Logger.debug("Ignoring non-ockam message #{inspect(message)} in handshake stage")
  #   {:ok, state}
  # end
  # def handle_non_message(message, %{stage: :data} = state) do
  #   RoutingSession.handle_data_message(message, state)
  # end

  def handle_init_message(init_message, state) do
    case parse_init_message(init_message) do
      {:ok, [initiator_route: initiator_route, handshake_data: handshake_data]} ->
        switch_to_data_stage(initiator_route, handshake_data, state)

      {:error, err} ->
        {:error, err}
    end
  end

  @doc """
  Parse init message into module start options

  Used in Ockam.Session.Spawner
  """
  def parse_init_message(message) do
    initiator_route = Message.return_route(message)
    handshake_data = Message.payload(message)
    {:ok, [initiator_route: initiator_route, handshake_data: handshake_data]}
  end

  defp switch_to_data_stage(initiator_route, handshake_data, state) do
    worker_mod = Map.fetch!(state, :worker_mod)
    worker_options = Map.fetch!(state, :worker_options)
    base_state = Map.fetch!(state, :base_state)

    worker_options =
      Keyword.merge(worker_options, route: initiator_route, handshake_data: handshake_data)

    case worker_mod.setup(worker_options, base_state) do
      {:ok, data_state} ->
        send_handshake_response(initiator_route, handshake_data, state)
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

  defp send_handshake_response(initiator_route, handshake_data, state) do
    msg = %{
      onward_route: initiator_route,
      return_route: [state.inner_address],
      payload: handshake_data
    }

    Ockam.Router.route(msg)
  end
end
