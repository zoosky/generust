use maud::{html, Markup};

use {{crate_name}}_core::Result;
use {{crate_name}}_service::RequestContext;

pub fn index(ctx: &RequestContext) -> Result<Markup> {
  let content = html! {
    div.uk-container {
      div.uk-section.uk-section-small {
        div.uk-container.uk-container-small {
          div.uk-text-center {
            h1.uk-heading-hero {
              "Welcome to {{project-name}}"
            }
          }
          (socket(ctx)?)
          (testbed_list(ctx)?)
        }
      }
    }
    script src=(ctx.router().route_static("client.js")?) defer? {}
  };
  Ok(html! {
    (crate::simple(ctx, "Home", content)?)
  })
}

fn socket(ctx: &RequestContext) -> Result<Markup> {
  Ok(crate::card(&ctx, html! {
    h4 { "WebSocket" }
    div {
      a href="" onclick=(crate::utils::onclick_event("send-ping", "", "")) { "Send Ping" }
    }
    div#socket-status { "Connecting..." }
    div#socket-results.uk-margin-top { }
  }))
}

fn testbed_list(ctx: &RequestContext) -> Result<Markup> {
  let ts = vec![
    ("dump", "Dump", "Displays a bunch of info about the app"),
    ("gallery", "Gallery", "Tests front-end components"),
    ("prototype", "Prototype", "A sandbox to play around in"),
    ("scroll", "Scroll", "Scrolling test"),
    ("crash", "Crash", "Simulates a server error"),
  ];
  Ok(crate::card(&ctx, html! {
    h4 { "Testbeds" }
    table.uk-table.uk-table-divider {
      tbody {
        @for t in ts {
          tr {
            td { a.(ctx.user_profile().link_class()) href=(ctx.router().route("testbed.detail", &[t.0])?) { (t.1) } }
            td { (t.2) }
          }
        }
      }
    }
  }))
}