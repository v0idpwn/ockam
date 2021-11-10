defmodule Ockam.Examples.Session.PipeChannel do
  @moduledoc """
  Pipe channel session example
  Using the session handshake module PipeChannel.Handshake
  to create a pipe channel
  """

  alias Ockam.Session.Separate, as: Session
  alias Ockam.Session.Spawner

  alias Ockam.Messaging.Delivery.ResendPipe
  alias Ockam.Messaging.PipeChannel

  def create_spawner() do
    spawner_options = [
      worker_mod: Session.Responder,
      worker_options: [
        worker_mod: PipeChannel.Simple,
        handshake: PipeChannel.Handshake,
        handshake_options: [
          pipe_mod: ResendPipe,
          sender_options: [confirm_timeout: 200]
        ]
      ]
    ]

    Spawner.create(spawner_options)
  end

  def create_responder() do
    responder_options = [
      worker_mod: PipeChannel.Simple,
      handshake: PipeChannel.Handshake,
      handshake_options: [
        pipe_mod: ResendPipe,
        sender_options: [confirm_timeout: 200]
      ]
    ]

    Session.Responder.create(responder_options)
  end

  def create_initiator(spawner_route) do
    Session.Initiator.create_and_wait(
      init_route: spawner_route,
      worker_mod: PipeChannel.Simple,
      worker_options: [],
      handshake: PipeChannel.Handshake,
      handshake_options: [
        pipe_mod: ResendPipe,
        sender_options: [confirm_timeout: 200]
      ]
    )
  end

  def run_local() do
    {:ok, responder} = create_responder()

    {:ok, responder_inner} = Ockam.AsymmetricWorker.get_inner_address(responder)

    {:ok, initiator} = create_initiator([responder_inner])

    Ockam.Node.register_address("me")

    Ockam.Router.route(%{
      onward_route: [initiator, "me"],
      return_route: ["me"],
      payload: "Hi me!"
    })

    :sys.get_state(Ockam.Node.whereis(initiator))
  end

  def run_local_with_spawner() do
    {:ok, spawner} = create_spawner()
    {:ok, initiator} = create_initiator([spawner])

    Ockam.Node.register_address("me")

    Ockam.Router.route(%{
      onward_route: [initiator, "me"],
      return_route: ["me"],
      payload: "Hi me!"
    })

    :sys.get_state(Ockam.Node.whereis(initiator))
  end
end
