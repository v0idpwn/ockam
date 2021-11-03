defmodule Ockam.Session.DataWorker do
  use Ockam.Worker

  @impl true
  def setup(options, state) do
    route = Keyword.get(options, :route)
    messages = Keyword.get(options, :messages)
    {:ok, Map.merge(state, %{route: route, messages: messages})}
  end

  def handle_message(message, state) do
    [_ | onward_route] = Message.onward_route(message)
    Ockam.Router.route(%{
      onward_route: state.route ++ onward_route,
      return_route: Message.return_route(message),
      payload: Message.payload(message)
    })
    {:ok, Map.update(state, :messages, [message], fn(messages) -> [message | messages] end)}
  end
end
