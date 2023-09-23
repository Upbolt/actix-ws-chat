use actix::{Actor, Addr};
use actix_files::NamedFile;
use actix_web::{
  cookie::Cookie,
  error::ErrorUnauthorized,
  get, post,
  web::{Data, Payload},
  App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use services::chat::Chat;

use crate::services::chat::{
  user::{User, Uuid},
  ChatEvent,
};

mod services;

#[get("/")]
async fn index() -> impl Responder {
  NamedFile::open(concat!(env!("CARGO_MANIFEST_DIR"), "/public/index.html"))
}

#[get("/chat")]
async fn stream_chat(
  req: HttpRequest,
  chat_service: Data<Addr<Chat>>,
  stream: Payload,
) -> impl Responder {
  let user = User::with_chat(chat_service.get_ref().clone());
  let id = user.id().clone();

  ws::start(user, &req, stream).map(|mut res| {
    res.add_cookie(
      &Cookie::build("id", id.as_str())
        .path("/")
        .http_only(true)
        .finish(),
    );
    res
  })
}

#[post("/message")]
async fn send_message(
  chat_service: Data<Addr<Chat>>,
  message: String,
  req: HttpRequest,
) -> actix_web::Result<impl Responder> {
  let id = req
    .cookie("id")
    .ok_or(ErrorUnauthorized("not authorized to send messages"))?;

  chat_service.do_send(ChatEvent::MessageSent {
    author_id: Uuid::from(id.value()),
    content: message,
  });

  Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
  let chat_service = Data::new(Chat::new().start());
  let app = move || {
    App::new()
      .app_data(chat_service.clone())
      .service(index)
      .service(stream_chat)
      .service(send_message)
  };

  Ok(HttpServer::new(app).bind(("0.0.0.0", 5050))?.run().await?)
}
