use async_trait::async_trait;
use ockam_core::{Encoded, Handler, Message, Worker};
use serde::{Deserialize, Serialize};
use serde_traitobject as st;

/// Make an envelope into a serialised byte stream
pub(crate) fn make<W, M>(m: M) -> Encoded
where
    W: Handler<M>,
    M: Message + Send,
{
    Envelope::<W>::new(m).encode().unwrap()
}

/// Take an envelope out of a serialised byte stream
pub(crate) fn take<W>(enc: &Encoded) -> Envelope<W>
where
    W: Worker,
{
    Message::decode(enc).unwrap()
}

/// A user message envelope addressed to a specific worker
#[derive(Serialize, Deserialize)]
pub struct Envelope<W: 'static>(
    // Does 'static break the lifetimes here?
    #[serde(with = "serde_traitobject")] Box<dyn EnvelopeProxy<W> + Send + 'static>,
);

impl<W: Worker> Message for Envelope<W> {}

impl<W: Worker> Envelope<W> {
    pub fn new<M>(msg: M) -> Self
    where
        W: Handler<M>,
        M: Message + Send + 'static,
    {
        Envelope(Box::new(EnvProxyImpl { inner: Some(msg) }))
    }
}

#[async_trait]
pub trait EnvelopeProxy<W: Worker>: st::Serialize + st::Deserialize {
    async fn handle(&mut self, _worker: &mut W, _ctx: &mut W::Context);
}

#[async_trait]
impl<W: Worker> EnvelopeProxy<W> for Envelope<W> {
    async fn handle(&mut self, worker: &mut W, ctx: &mut W::Context) {
        self.0.handle(worker, ctx).await
    }
}

/// Implementation of a message typed envelope proxy
#[derive(Serialize, Deserialize)]
pub struct EnvProxyImpl<M>
where
    M: Message + Send,
{
    #[serde(bound(deserialize = "M: Serialize"))]
    inner: Option<M>,
}

#[async_trait]
impl<W, M> EnvelopeProxy<W> for EnvProxyImpl<M>
where
    W: Worker + Handler<M>,
    M: Message + Send,
{
    async fn handle(&mut self, worker: &mut W, ctx: &mut W::Context) {
        <W as Handler<M>>::handle(worker, self.inner.take().unwrap(), ctx).await
    }
}

pub trait ToEnvelope<W, M>
where
    W: Worker + Handler<M>,
    M: Message + Send,
{
    fn pack(msg: M) -> Envelope<W> {
        Envelope::new(msg)
    }
}
