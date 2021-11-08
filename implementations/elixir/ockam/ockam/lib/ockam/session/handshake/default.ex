defmodule Ockam.Session.Handshake.Default do
  @moduledoc """
  Simple handshake logic using an empty message
  and return route tracing
  """

  @behaviour Ockam.Session.Handshake

  alias Ockam.Message

  @spec init(Keyword.t(), map()) :: {:ok, Message.t(), map()}
  def init(_options, state) do
    init_route = Map.fetch!(state, :init_route)

    {:ok,
     %{
       onward_route: init_route,
       return_route: [state.inner_address],
       payload: ""
     }, state}
  end

  def handle_initiator(_options, message, state) do
    data_route = Message.return_route(message)
    handshake_data = Message.payload(message)
    ## TODO: use special data types?
    case handshake_data == "" do
      true ->
        {:ok, [route: data_route], state}

      false ->
        {:error, {:invalid_handshake_message, message}}
    end
  end

  def handle_responder(_options, message, state) do
    initiator_route = Message.return_route(message)
    handshake_data = Message.payload(message)

    response = %{
      onward_route: initiator_route,
      return_route: [state.inner_address],
      payload: handshake_data
    }

    {:ok, response, [route: initiator_route], state}
  end
end
