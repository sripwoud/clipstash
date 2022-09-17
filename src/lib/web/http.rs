use rocket::form::{Contextual, Form};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket::response::{status, Redirect};
use rocket::{uri, State};
use rocket::http::uri::fmt::UriQueryArgument::Raw;

use crate::data::Database;
use crate::{Db, service};
use crate::service::action;
use crate::web::{form, renderer::Renderer, PageError, ctx};
use crate::{ServiceError, ShortCode};
use crate::domain::clip::field::Content;
use crate::web::ctx::*;

#[rocket::get("/")]
fn home(renderer: &State<Renderer<'_>>) -> RawHtml<String> {
    let ctx = Home::default();
    RawHtml(renderer.render(ctx, &[]))
}


#[rocket::post("/", data = "<form>")]
pub async fn add_clip(
    // rocket can only call a function for a route if all data exists.
    // form data may not exist or be incorrect
    // using Contextual allows to accept invalid form data
    form: Form<Contextual<'_, form::NewClip>>,
    db: &State<Db>,
    renderer: &State<Renderer<'_>>,
) -> Result<Redirect, (Status, RawHtml<String>)> {
    let form = form.into_inner(); // to get Contextual
    if let Some(value) = form.value {
        let req = service::ask::NewClip {
            // these values comes from from::NewClip, which uses field::*, so we know all these fields are already validated
            content: value.content,
            title: value.title,
            expires: value.expires,
            password: value.password,
        };

        match action::new_clip(req, db.get_pool()).await {
            Ok(clip) => Ok(Redirect::to(uri!(get_clip(shortcode = clip.shortcode)))),
            Err(e) => {
                eprintln!("internal error: {:?}", e);
                Err((
                    Status::InternalServerError,
                    RawHtml(
                        renderer.render(ctx::Home::default(),
                                        &["A server error occurred. Please try again."]
                        )
                    )
                ))
            }
        }
    } else {
        let errors = form
            .context
            .errors()
            .map(|err| {
                use rocket::form::error::ErrorKind;
                if let ErrorKind::Validation(msg) = &err.kind {
                    msg.as_ref()
                } else {
                    eprintln!("unhandled error: {:?}", err);
                    "An error occurred, please try again"
                }
            })
            .collect::<Vec<_>>();

        Err((
            Status::BadRequest,
            RawHtml(
                renderer.render_with_data(
                    ctx::Home::default(),
                    ("clip", &form.context),
                    &errors
                )
            )
        ))
    }
}

#[rocket::get("/clip/<shortcode>")]
pub async fn get_clip(
    shortcode: ShortCode,
    db: &State<Db>,
    renderer: &State<Renderer<'_>>,
) -> Result<status::Custom<RawHtml<String>>, PageError> {
    fn render_with_status<T: PageCtx + serde::Serialize + std::fmt::Debug>(
        status: Status,
        context: T,
        renderer: &Renderer,
    ) -> Result<status::Custom<RawHtml<String>>, PageError> {
        Ok(status::Custom(status, RawHtml(renderer.render(context, &[]))))
    }

    match action::get_clip(shortcode.clone().into(), db.get_pool()).await {
        Ok(clip) => {
            let context = ctx::ViewClip::new(clip);
            render_with_status(Status::Ok, context, renderer)
        }
        Err(e) => match e {
            ServiceError::PermissionError(_) => {
                let context = ctx::PasswordRequired::new(shortcode);
                render_with_status(Status::Unauthorized, context, renderer)
            }
            ServiceError::NotFound => Err(PageError::NotFound("Clip not found".to_owned())),
            _ => Err(PageError::Internal("server error".to_owned()))
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![home, get_clip, add_clip]
}

pub mod catcher {
    use rocket::Request;
    use rocket::{catch, catchers, Catcher};

    #[catch(default)]
    fn default(req: &Request) -> &'static str {
        eprintln!("General error: {:?}", req);
        "Something went wrong"
    }

    #[catch(500)]
    fn internal_error(req: &Request) -> &'static str {
        eprintln!("Internal error: {:?}", req);
        "internal server error"
    }

    #[catch(404)]
    fn not_found() -> &'static str {
        "404"
    }

    pub fn catchers() -> Vec<Catcher> {
        catchers![not_found, default, internal_error]
    }
}


