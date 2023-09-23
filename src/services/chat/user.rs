use actix::{
  Actor, ActorContext, Addr, AsyncContext, Handler, Message, Recipient, Running, StreamHandler,
};
use actix_web_actors::ws;
use derive_more::{Deref, Display};

use super::{Chat, ChatEvent};

#[derive(Deref, Clone, Display)]
pub(crate) struct Uuid(String);

#[derive(Clone)]
pub(crate) struct User {
  id: Uuid,
  chat: Addr<Chat>,
  connection: Option<Recipient<ChatMessage>>,
}

#[derive(Message, Clone)]
#[rtype(default = "()")]
pub(crate) struct ChatMessage(String);

impl Into<ChatMessage> for String {
  fn into(self) -> ChatMessage {
    ChatMessage(self)
  }
}

impl From<&str> for Uuid {
  fn from(value: &str) -> Self {
    Self(value.to_string())
  }
}

impl User {
  pub(crate) fn with_chat(chat: Addr<Chat>) -> Self {
    Self {
      id: Uuid(uuid::Uuid::new_v4().as_hyphenated().to_string()),
      chat,
      connection: None,
    }
  }

  pub(crate) fn send(&self, message: impl Into<String>) {
    if let Some(connection) = &self.connection {
      let message: String = message.into();

      connection.do_send(message.into());
    }
  }

  pub(crate) fn id(&self) -> &Uuid {
    &self.id
  }
}

impl Actor for User {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.connection = Some(ctx.address().recipient());
    self.chat.do_send(ChatEvent::UserJoined(self.clone()));
  }

  fn stopping(&mut self, _: &mut Self::Context) -> Running {
    self.chat.do_send(ChatEvent::UserLeft(self.id().clone()));
    Running::Stop
  }
}

impl Handler<ChatMessage> for User {
  type Result = ();

  fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) -> Self::Result {
    ctx.text(msg.0);
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for User {
  fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match message {
      Ok(ws::Message::Text(text)) => self.chat.do_send(ChatEvent::MessageSent {
        author_id: self.id.clone(),
        content: text.to_string(),
      }),
      Ok(ws::Message::Close(reason)) => {
        ctx.close(reason);
        ctx.stop();
      }
      _ => (),
    }
  }
}
