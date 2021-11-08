defmodule Ockam.Messaging.PipeChannel do
  @moduledoc """
  Ockam channel using pipes to deliver messages

  Can be used with different pipe implementations to get different delivery properties

  See `Ockam.Messaging.PipeChannel.Initiator` and `Ockam.Messaging.PipeChannel.Responder` for usage

  Session setup:

  See `Ockam.Messaging.PipeChannel.Handshake`

  Message forwarding:

  Each channel endpoint is using two addresses: INNER and OUTER.
  INNER address us used to communicate with the pipes
  OUTER address is used to communicate to other workers

  On receiving a message from OUTER address with:
  OR: [outer] ++ onward_route
  RR: return_route

  Channel endpoint sends a message with:
  OR: [local_sender, remote_endpoint] ++ onward_route
  RR: return_route

  On receiving a message from INNER address with:
  OR: [inner] ++ onward_route
  RR: return_route

  It forwards a message with:
  OR: onward_route
  RR: [outer] ++ return_route
  """

  alias Ockam.Message
  alias Ockam.Router

  @doc false
  ## Inner message is forwarded with outer address in return route
  def forward_inner(message, state) do
    [_me | onward_route] = Message.onward_route(message)
    return_route = Message.return_route(message)
    payload = Message.payload(message)

    Router.route(%{
      onward_route: onward_route,
      return_route: [state.address | return_route],
      payload: payload
    })
  end

  @doc false
  ## Outer message is forwarded through sender
  ## to other channel endpoints inner address
  def forward_outer(message, state) do
    channel_route = Map.fetch!(state, :channel_route)

    [_me | onward_route] = Message.onward_route(message)
    return_route = Message.return_route(message)
    payload = Message.payload(message)

    sender = Map.fetch!(state, :sender)

    Router.route(%{
      onward_route: [sender | channel_route ++ onward_route],
      return_route: return_route,
      payload: payload
    })
  end

  @doc false
  def register_inner_address(state) do
    {:ok, inner_address} = Ockam.Node.register_random_address()
    Map.put(state, :inner_address, inner_address)
  end
end

defmodule Ockam.Messaging.PipeChannel.Metadata do
  @moduledoc """
  Encodable data structure for pipechannel handshake metadata

  `receiver_route` - local route to receiver worker
  `channel_route` - local route to the channel worker (inner address)
  """

  defstruct [:receiver_route, :channel_route]

  @type t() :: %__MODULE__{}

  ## TODO: use proper address encoding
  @schema {:struct, [receiver_route: {:array, :data}, channel_route: {:array, :data}]}

  @spec encode(t()) :: binary()
  def encode(meta) do
    :bare.encode(meta, @schema)
  end

  @spec decode(binary()) :: t()
  def decode(data) do
    case :bare.decode(data, @schema) do
      {:ok, meta, ""} ->
        struct(__MODULE__, meta)

      other ->
        exit({:meta_decode_error, data, other})
    end
  end
end

defmodule Ockam.Messaging.PipeChannel.Simple do
  @moduledoc """
  Simple implementation of pipe channel.
  Does not manage the session.
  Requires a known address to the local pipe sender and remote channel end

  Using two addresses for inner and outer communication.

  forwards messages from outer address to the sender and remote channel
  forwards messages from inner address to the onward route and traces own outer address in the return route

  Options:

  `sender` - address of the sender worker
  `channel_route` - route from remote receiver to remote channel end
  """
  use Ockam.AsymmetricWorker

  alias Ockam.Messaging.PipeChannel

  @impl true
  def inner_setup(options, state) do
    sender = Keyword.fetch!(options, :sender)
    channel_route = Keyword.fetch!(options, :channel_route)

    {:ok, Map.merge(state, %{sender: sender, channel_route: channel_route})}
  end

  @impl true
  def handle_inner_message(message, state) do
    PipeChannel.forward_inner(message, state)
    {:ok, state}
  end

  @impl true
  def handle_outer_message(message, state) do
    PipeChannel.forward_outer(message, state)
    {:ok, state}
  end
end

defmodule Ockam.Messaging.PipeChannel.Initiator do
  @moduledoc """
  Pipe channel initiator.

  A session initiator using `Ockam.Messaging.PipeChannel.Handshake` for handshake
  and `Ockam.Messaging.PipeChannel.Simple` for data exchange

  Options:

  `spawner_route` - init route for the session
  `pipe_mod` - pipe module
  `sender_options` - options for sender
  `receiver_options` - options for receiver
  """

  alias Ockam.Messaging.PipeChannel

  alias Ockam.Session.Routing.Pluggable, as: Session

  def create(options) do
    ## TODO: rename to init_route
    spawner_route = Keyword.fetch!(options, :spawner_route)

    pipe_mod = Keyword.fetch!(options, :pipe_mod)
    sender_options = Keyword.get(options, :sender_options, [])
    receiver_options = Keyword.get(options, :receiver_options, [])

    Session.Initiator.create(
      init_route: spawner_route,
      worker_mod: PipeChannel.Simple,
      worker_options: [],
      handshake: PipeChannel.Handshake,
      handshake_options: [
        pipe_mod: pipe_mod,
        sender_options: sender_options,
        receiver_options: receiver_options
      ]
    )
  end

  ## TODO: solve duplication with Session.Initiator.create_and_wait
  def create_and_wait(options, interval \\ 50, timeout \\ 5000) do
    with {:ok, address} <- create(options),
         :ok <- Session.Initiator.wait_for_session(address, interval, timeout) do
      {:ok, address}
    end
  end
end

defmodule Ockam.Messaging.PipeChannel.Responder do
  @moduledoc """
  Pipe channel responder

  A session responder using `Ockam.Messaging.PipeChannel.Handshake` for handshake
  and `Ockam.Messaging.PipeChannel.Simple` for data exchange

  Options:

  `pipe_mod` - pipe module
  `sender_options` - options for sender
  `receiver_options` - options for receiver
  """

  alias Ockam.Messaging.PipeChannel

  def create(options) do
    init_message = Keyword.get(options, :init_message)

    pipe_mod = Keyword.fetch!(options, :pipe_mod)
    sender_options = Keyword.get(options, :sender_options, [])
    receiver_options = Keyword.get(options, :receiver_options, [])

    Ockam.Session.Routing.Pluggable.Responder.create(
      init_message: init_message,
      worker_mod: PipeChannel.Simple,
      handshake: PipeChannel.Handshake,
      handshake_options: [
        pipe_mod: pipe_mod,
        sender_options: sender_options,
        receiver_options: receiver_options
      ]
    )
  end
end

defmodule Ockam.Messaging.PipeChannel.Spawner do
  @moduledoc """
  Pipe channel receiver spawner

  On message spawns a channel receiver
  with remote route as a remote receiver route
  and channel route taken from the message metadata

  Options:

  `responder_options` - additional options to pass to the responder
  """

  def create(options) do
    responder_options = Keyword.fetch!(options, :responder_options)

    Ockam.Session.Spawner.create(
      worker_mod: Ockam.Messaging.PipeChannel.Responder,
      worker_options: responder_options
    )
  end
end
