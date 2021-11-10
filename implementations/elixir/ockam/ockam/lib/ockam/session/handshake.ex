defmodule Ockam.Session.Handshake do
  @moduledoc """
  Session handshake behaviour.

  Used in `Ockam.Session.Routing.Pluggable` modules
  """
  @type message() :: Ockam.Message.t()

  @type handshake_state() :: %{
          :init_route => Ockam.Address.route(),
          :outer_address => Ockam.Address.t(),
          :inner_address => Ockam.Address.t(),
          any() => any()
        }

  @callback init(options :: Keyword.t(), state :: handshake_state()) ::
              {:ok, message(), state :: handshake_state()} | {:error, any()}

  ## TODO: non-terminating handshake support
  ## TODO: error result
  ## TODO: explicit structure fields in result
  @callback handle_initiator(
              handshake_options :: Keyword.t(),
              message :: message(),
              state :: handshake_state()
            ) ::
              {:ok, worker_options :: Keyword.t(), state :: handshake_state()}
  @callback handle_responder(
              handshake_options :: Keyword.t(),
              message :: message(),
              state :: handshake_state()
            ) ::
              {:ok, response :: message(), worker_options :: Keyword.t(),
               state :: handshake_state()}
end
