defmodule Ockam.Session.Separate do
end

defmodule Ockam.Session.Separate.Initiator do
  use Ockam.AsymmetricWorker

  alias Ockam.AsymmetricWorker

  alias Ockam.Message
  alias Ockam.Router

  alias Ockam.Session.PreWorker

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

      _other ->
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

  def get_data_worker(session_worker) do
    Ockam.Worker.call(session_worker, :get_data_worker)
  end

  @impl true
  def handle_call(:get_stage, _from, state) do
    {:reply, Map.get(state, :stage), state}
  end

  def handle_call(:get_data_worker, _from, state) do
    {:reply, Map.fetch!(state, :data_worker), state}
  end

  @impl true
  def inner_setup(options, state) do
    init_route = Keyword.fetch!(options, :init_route)

    ## rename to data_mod
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    handshake = Keyword.get(options, :handshake, Ockam.Session.Handshake.Default)
    handshake_options = Keyword.get(options, :handshake_options, [])

    {:ok, pre_worker} = PreWorker.create(worker_mod: worker_mod, worker_options: worker_options)
    {:ok, pre_worker_inner} = AsymmetricWorker.get_inner_address(pre_worker)

    hanshake_state = %{
      init_route: init_route,
      outer_address: pre_worker,
      inner_address: pre_worker_inner,
      handshake_address: state.inner_address
    }

    hanshake_state = send_handshake(handshake, handshake_options, hanshake_state)

    state =
      Map.merge(state, %{
        hanshake_state: hanshake_state,
        handshake: handshake,
        handshake_options: handshake_options,
        data_worker: pre_worker
      })

    {:ok, Map.put(state, :stage, :handshake)}
  end

  def send_handshake(handshake, handshake_options, hanshake_state) do
    {:ok, handshake_msg, hanshake_state} = handshake.init(handshake_options, hanshake_state)
    Logger.info("handshake_msg: #{inspect(handshake_msg)}")
    Ockam.Router.route(handshake_msg)
    hanshake_state
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

  def handle_message(message, %{stage: :data} = state) do
    case message_type(message, state) do
      :outer ->
        data_worker = Map.fetch!(state, :data_worker)
        [_me | onward_route] = Message.onward_route(message)

        Ockam.Router.route(%{
          onward_route: [data_worker | onward_route],
          return_route: Message.return_route(message),
          payload: Message.payload(message)
        })

      _ ->
        Logger.warn("Ignoring message in data stage: #{inspect(message)}. Not implemented")
    end

    {:ok, state}
  end

  def handle_handshake_message(message, state) do
    handshake = Map.fetch!(state, :handshake)
    handshake_options = Map.fetch!(state, :handshake_options)
    hanshake_state = Map.fetch!(state, :hanshake_state)

    case handshake.handle_initiator(handshake_options, message, hanshake_state) do
      {:ok, options, hanshake_state} ->
        switch_to_data_stage(options, hanshake_state, state)

      {:error, err} ->
        ## TODO: error handling in Ockam.Worker
        {:error, err}
    end
  end

  def switch_to_data_stage(options, hanshake_state, state) do
    pre_worker = Map.fetch!(state, :data_worker)

    case PreWorker.start(pre_worker, options) do
      :ok ->
        Logger.info("Worker started: #{inspect(pre_worker)}")
        {:ok, Map.merge(state, %{hanshake_state: hanshake_state, stage: :data})}

      {:error, err} ->
        {:stop, {:cannot_start_data_worker, {:error, err}, options, hanshake_state}, state}
    end
  end
end

defmodule Ockam.Session.Separate.Responder do
  use Ockam.AsymmetricWorker

  alias Ockam.AsymmetricWorker

  alias Ockam.Message

  alias Ockam.Session.PreWorker

  require Logger

  def get_data_worker(session_worker) do
    Ockam.Worker.call(session_worker, :get_data_worker)
  end

  @impl true
  def address_prefix(_options), do: "S_S_R_"

  @impl true
  def inner_setup(options, state) do
    worker_mod = Keyword.fetch!(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    handshake = Keyword.get(options, :handshake, Ockam.Session.Handshake.Default)
    handshake_options = Keyword.get(options, :handshake_options, [])

    {:ok, pre_worker} = PreWorker.create(worker_mod: worker_mod, worker_options: worker_options)
    {:ok, pre_worker_inner} = AsymmetricWorker.get_inner_address(pre_worker)

    hanshake_state = %{
      outer_address: pre_worker,
      inner_address: pre_worker_inner,
      handshake_address: state.inner_address
    }

    state =
      Map.merge(state, %{
        stage: :handshake,
        hanshake_state: hanshake_state,
        handshake: handshake,
        handshake_options: handshake_options,
        data_worker: pre_worker
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
  def handle_call(:get_data_worker, _from, state) do
    {:reply, Map.fetch!(state, :data_worker), state}
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
    Logger.debug("Ignoring message in data stage: #{inspect(message)}. Not implemented")
    {:ok, state}
  end

  def handle_handshake_message(message, state) do
    handshake = Map.fetch!(state, :handshake)
    handshake_options = Map.fetch!(state, :handshake_options)

    hanshake_state = Map.fetch!(state, :hanshake_state)

    case handshake.handle_responder(handshake_options, message, hanshake_state) do
      {:ok, response, options, hanshake_state} ->
        switch_to_data_stage(response, options, hanshake_state, state)

      {:error, err} ->
        {:error, err}
    end
  end

  defp switch_to_data_stage(response, handshake_options, hanshake_state, state) do
    pre_worker = Map.fetch!(state, :data_worker)

    case PreWorker.start(pre_worker, handshake_options) do
      :ok ->
        Logger.info("handshake response: #{inspect(response)}")
        send_handshake_response(response)
        ## TODO: use data_state instead of state futher on?
        {:ok, Map.merge(state, %{hanshake_state: hanshake_state, stage: :data})}

      {:error, err} ->
        worker_mod = Map.fetch!(state, :worker_mod)

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
