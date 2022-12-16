use yew::prelude::*;
use common::{Board, BoardSpace, PlayerNum, DeckRng, GameState};

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: Board
}

#[function_component(Battle)]
pub fn battle() -> Html {
    let board = GameState::<DeckRng>::default().board().clone();
    html! {
        <section id="page">
            <BoardComponent board={board}/>
            <div class={classes!("choices")}>
                <div class={classes!("card")}>
                    <div>{"Splattershot"}</div>
                    <div class={classes!("card-grid")}>
                        {"blah"}
                    </div>
                    <div>{"Priority: 6"}</div>
                    <div>{"Special cost: 3"}</div>
                </div>
                <div class={classes!("card")}>
                    <div>{"Flyfish"}</div>
                    <div class={classes!("card-grid")}>
                        {"Blah"}
                    </div>
                    <div>{"Priority: 13"}</div>
                    <div>{"Special cost: 5"}</div>
                </div>
                <div class={classes!("card")}>
                    <div>{"Splatterscope"}</div>
                    <div class={classes!("card-grid")}>
                        {"Blah"}
                    </div>
                    <div>{"Priority: 9"}</div>
                    <div>{"Special cost: 3"}</div>
                </div>
                <div class={classes!("card")}>
                    <div>{"Splat Bomb"}</div>
                    <div class={classes!("card-grid")}>
                        {"Blah"}
                    </div>
                    <div>{"Priority: 3"}</div>
                    <div>{"Special cost: 1"}</div>
                </div>
                <button>{"Pass"}</button>
                <button>{"Special"}</button>
            </div>
            <div class={classes!("timer")}>
                <div>{"Turns left: 12"}</div>
                <div>{"Time remaining: 1:30"}</div> 
            </div>
            <div class={classes!("special-gauge")}>{"Special gauge: 0"}</div>
            <button class={classes!("deck")}>{"View deck"}</button>
        </section>
    }
}

#[function_component(BoardComponent)]
pub fn board(props: &BoardProps) -> Html {
    let width = props.board.width();
    let height = props.board.height();
    let spaces = props.board.get();
    html! {
        <div class={classes!("board")}>
            <div 
                class={classes!("board-grid")}
                style={format!("display: grid; grid-template-rows: repeat({}, 1fr); grid-template-columns: repeat({}, 1fr)", height, width)}>
                {
                    spaces.iter().flatten().map(|s| view_space(s)).collect::<Html>()
                }
            </div>
        </div>
    }
}

fn view_space(space: &BoardSpace) -> Html {
    html! {
        <div class={match space {
            BoardSpace::Empty => classes!("empty"),
            BoardSpace::Ink { player_num } => {
                classes!(get_player_num_class(player_num), "ink")
            }
            BoardSpace::Special { player_num, is_activated } => {
                get_special_classes(player_num, *is_activated)
            }
            BoardSpace::Wall => classes!("wall"),
            BoardSpace::OutOfBounds => classes!("oob"),
        }}></div>
    }
}

fn get_player_num_class(player_num: &PlayerNum) -> String {
    match player_num {
        PlayerNum::P1 => "p1".to_string(),
        PlayerNum::P2 => "p2".to_string(),
    }
}

fn get_special_classes(player_num: &PlayerNum, is_activated: bool) -> Classes {
    if is_activated {
        classes!(get_player_num_class(player_num), "special", "activated")
    } else {
        classes!(get_player_num_class(player_num), "special")
    }
} 
