use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use futures::channel::mpsc::Sender;
use std::cell::RefCell;
use std::rc::Rc;

mod battle;
mod ws;
mod event_bus;

pub type User = Rc<UserInner>;

#[derive(Debug, PartialEq)]
pub struct UserInner {
    pub user_id: RefCell<String>
}

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[at("/battle")]
    Battle,
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Login /> },
        Route::Battle => html! { <battle::Battle /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    let ctx = use_state(|| {
        Rc::new(UserInner {
            user_id: RefCell::new("initial".to_string())
        })
    });
    html! {
        <ContextProvider<User> context={(*ctx).clone()}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<User>>
    }
}

#[function_component(Login)]
pub fn login() -> Html {
    let state = use_state(|| String::new());
    let user = use_context::<User>().unwrap();
    let oninput = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            state.set(e.target_unchecked_into::<HtmlInputElement>().value())
        })
    };
    let onclick = {
        let state = state.clone();
        let user = user.clone();
        Callback::from(move |_| {
            *user.user_id.borrow_mut() = (*state).clone()
        })
    };
    html! {
        <div class="bg-gray-800 flex w-screen">
            <div class="container mx-auto flex flex-col justify-center items-center">
                <form class="m-4 flex">
                    <input oninput={oninput} placeholder="User ID"/>
                    <Link<Route> to={Route::Battle}> 
                        <button onclick={onclick} disabled={state.len() < 1}>
                            {"Start Battle"}
                        </button>
                    </Link<Route>>
                </form>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
