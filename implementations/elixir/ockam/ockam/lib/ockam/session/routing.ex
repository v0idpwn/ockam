defmodule Ockam.Session.Routing do
  @moduledoc """
  Simple routing session logic.
  Initiator sends an empty message to the spawner on start
  and waits for a response from the responder.
  """
end


## TODO asymmetric worker for session
## The main issue is that if internal worker is asymmetric
## it might override the internal address


## First option: non-buffering session, messages are dropped if the session is not ready yet
defmodule Ockam.Session.Routing.Initiator do
  use Ockam.Worker

  alias Ockam.Message
  alias Ockam.Router

  require Logger

  @impl true
  def setup(options, state) do
    init_route = Keyword.fetch!(options, :init_route)

    base_state = state
    ## rename to data_mod
    worker_mod = Keyword.fetch(options, :worker_mod)
    worker_options = Keyword.get(options, :worker_options, [])

    state = Map.merge(state, %{init_route: init_route, worker_options: worker_options, base_state: base_state})
    state = send_handshake(state)
    {:ok, Map.put(state, :stage, :handshake)}
  end

  @impl true
  def handle_message(message, %{stage: :handshake} = state) do
    case is_handshake(message) do
      true ->
        handle_handshake_message(message, state)
      false ->
        Logger.debug("Ignoring message #{inspect(message)} in handshake stage")
        {:ok, state}
    end
  end

  def handle_message(message, %{stage: :data} = state) do
    handle_data_message(message, state)
  end

  def handle_handshake_message(message, state) do
    data_route = Message.return_route(message)
    handshake_data = Message.payload(message)

    {:ok, switch_to_data_stage(data_route, handshake_data, state)}
  end

  def switch_to_data_stage(data_route, handshake_data, state) do
    base_state = Map.get(state, :base_state)
    worker_mod = Map.fetch!(state, :worker_mod)
    worker_options = Map.fetch!(state, :worker_options)

    options = Keyword.merge(worker_options, [route: data_route, handshake_data: handshake_data])

    worker_mod.setup(options, base_state)
  end

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

  def send_handshake(state) do
    init_route = Map.fetch!(state, :init_route)
    Router.route(%{
      onward_route: init_route,
      return_route: state.address,
      payload: ""
    })
    state
  end
end
