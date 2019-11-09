use maud::{html, Markup};

use {{crate_name}}_core::Result;
use {{crate_name}}_service::RequestContext;

pub fn settings(ctx: &RequestContext) -> Result<Markup> {
  let content = html! {
    div.uk-section.uk-section-small {
      div.uk-container {
        div.uk-text-center {
          h1.uk-heading-hero {
            "Settings"
          }
          div.uk-text-lead {
            "A work in progress"
          }
          div.uk-margin-top {
            table.uk-table.uk-table-divider.uk-text-left {
              tbody {
                tr {
                  th { "Config Directory" }
                  td { (ctx.app().files().cfg_dir()) }
                }
              }
            }
          }
        }
      }
    }
  };
  Ok(html! {
    (crate::simple(ctx, "Settings", content)?)
  })
}