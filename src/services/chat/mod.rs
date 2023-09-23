use actix::{Actor, Context, Handler, Message};

use self::user::{User, Uuid};

pub(crate) mod user;

#[derive(Default)]
pub(crate) struct Chat {
  users: Vec<User>,
}

impl Chat {
  pub(crate) fn new() -> Self {
    Self::default()
  }
}

impl Actor for Chat {
  type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) enum ChatEvent {
  UserJoined(User),
  UserLeft(Uuid),
  MessageSent { author_id: Uuid, content: String },
}

impl Handler<ChatEvent> for Chat {
  type Result = ();

  fn handle(&mut self, msg: ChatEvent, _: &mut Self::Context) -> Self::Result {
    let message = match msg {
      ChatEvent::UserJoined(user) => {
        let message = format!("!! << {} joined the chat >> !!", user.id());
        self.users.push(user);

        message
      }
      ChatEvent::UserLeft(id) => format!("{} left the chat", id),
      ChatEvent::MessageSent { author_id, content } => format!("{}: {}", author_id, content),
    };

    for user in self.users.iter() {
      user.send(&message);
    }
  }
}
