use crate::websocket_msg::{SendResponseMessage, ServerSender};

use {{crate_name}}_core::{RequestMessage, ResponseMessage, Result};
use {{crate_name}}_service::handler::MessageHandler;
use {{crate_name}}_service::AppConfig;

use actix::{Actor, AsyncContext, StreamHandler};
use actix_session::Session;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::WebsocketContext;
use actix_web_actors::ws::{Message, ProtocolError};

#[derive(derive_more::Constructor)]
pub(crate) struct ServerSocket {
  binary: bool,
  handler: MessageHandler
}

impl ServerSocket {
  fn handler(&self) -> &MessageHandler {
    &self.handler
  }

  fn handle_text(&self, s: String, wsc: &mut WebsocketContext<Self>) -> Result<()> {
    let req = RequestMessage::from_json(&s)?;
    self.handle_message(req, wsc)
  }

  fn handle_binary(&self, bytes: bytes::Bytes, wsc: &mut WebsocketContext<Self>) -> Result<()> {
    let b: &[u8] = bytes.as_ref();
    let req = RequestMessage::from_binary(&b.to_vec())?;
    self.handle_message(req, wsc)
  }

  fn handle_message(&self, req: RequestMessage, wsc: &mut WebsocketContext<Self>) -> Result<()> {
    for msg in self.handler.on_message(req)? {
      self.send_ws(&msg, wsc)?;
    }
    Ok(())
  }

  fn handle_error(&self, e: &{{crate_name}}_core::Error, wsc: &mut WebsocketContext<Self>) {
    slog::warn!(&self.handler().log(), "Error handling message: {}", e);
    let msg = ResponseMessage::ServerError {
      reason: format!("{}", e),
      content: "Error handling message".into()
    };
    match self.send_ws(&msg, wsc) {
      Ok(_) => (),
      Err(e) => slog::warn!(&self.handler().log(), "Error sending server error message: {}", e)
    }
  }

  fn send_ws(&self, rsp: &ResponseMessage, wsc: &mut WebsocketContext<Self>) -> Result<()> {
    if self.binary {
      wsc.binary(rsp.to_binary()?)
    } else {
      wsc.text(rsp.to_json()?)
    }
    Ok(())
  }
}

impl Actor for ServerSocket {
  type Context = WebsocketContext<Self>;

  fn started(&mut self, wsc: &mut Self::Context) {
    {
      let sender = Box::new(ServerSender::new(wsc.address()));
      let mut connections = self.handler.ctx().app().connections().write().unwrap();
      connections.add::<ServerSender>(self.handler.channel_id(), *self.handler().connection_id(), sender);
    }
    let msgs = match self.handler.on_open() {
      Ok(m) => m,
      Err(e) => {
        slog::error!(self.handler.log(), "Unable to process on_open: {}", e);
        vec![]
      }
    };
    for msg in msgs {
      match self.send_ws(&msg, wsc) {
        Ok(_) => (),
        Err(e) => slog::warn!(self.handler.log(), "Unable to send initial open messages: {}", e)
      }
    }
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
    let mut connections = self.handler.ctx().app().connections().write().unwrap();
    connections.remove(self.handler.channel_id(), *self.handler().connection_id());
    actix::Running::Stop
  }
}

impl actix::Handler<SendResponseMessage> for ServerSocket {
  type Result = ();

  fn handle(&mut self, m: SendResponseMessage, ctx: &mut Self::Context) {
    match self.send_ws(m.msg(), ctx) {
      Ok(_) => (), // noop for now
      Err(e) => self.handle_error(&{{crate_name}}_core::Error::from(format!("Error sending message [{:?}]: {}", m.msg(), e)), ctx)
    }
  }
}

impl StreamHandler<Message, ProtocolError> for ServerSocket {
  fn handle(&mut self, msg: Message, wsc: &mut Self::Context) {
    match msg {
      Message::Ping(msg) => wsc.pong(&msg),
      Message::Text(text) => match &self.handle_text(text, wsc) {
        Ok(_) => (),
        Err(e) => self.handle_error(&e, wsc)
      },
      Message::Binary(bin) => match self.handle_binary(bin, wsc) {
        Ok(_) => (),
        Err(e) => self.handle_error(&e, wsc)
      },
      _ => ()
    }
  }
}

/// Available at `/s/{key}/connect` (WebSocket handler)`
pub fn connect(
  session: Session, cfg: web::Data<AppConfig>, key: web::Path<String>, req: HttpRequest, stream: web::Payload
) -> std::result::Result<HttpResponse, Error> {
  let ctx = crate::req_context(&session, &cfg, "connect");
  let binary = match req.query_string() {
    x if x.contains("t=b") => true,
    x if x.contains("t=j") => false,
    _ => !cfg.verbose()
  };

  let id = uuid::Uuid::new_v4();
  let handler = MessageHandler::new(id, key.clone(), ctx);
  let socket = ServerSocket::new(binary, handler);
  actix_web_actors::ws::start(socket, &req, stream)
}
