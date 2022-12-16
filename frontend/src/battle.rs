use yew::prelude::*;
use common::{Board, BoardSpace, Card, CardSpace, InkSpace, PlayerNum, DeckRng, GameState};

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: Board
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card: Card
}

#[function_component(Battle)]
pub fn battle() -> Html {
    let board = GameState::<DeckRng>::default().board().clone();
    let e: CardSpace = None;
    let i: CardSpace = Some(InkSpace::Normal);
    let s: CardSpace = Some(InkSpace::Special);
    let card1 = Card::new(
        "Splattershot".to_string(),
        8,
        [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, i, i, s, e, e, e],
            [e, e, i, i, i, i, e, e],
            [e, e, i, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ],
        3,
    );

    let card2 = Card::new(
        "Slosher".to_string(),
        6,
        [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, i, e, e, e, e, e],
            [e, e, e, s, i, e, e, e],
            [e, e, i, i, i, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ],
        3,
    );

    let card3 = Card::new(
        "Zapfish".to_string(),
        9,
        [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, i, e, e],
            [e, e, e, i, i, e, e, e],
            [e, e, e, i, s, i, e, e],
            [e, e, i, e, i, i, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ],
        4,
    );
    let card4 = Card::new(
        "Blaster".to_string(),
        8,
        [
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, i, e, e, i, s, e, e],
            [e, e, i, i, i, i, e, e],
            [e, e, i, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
            [e, e, e, e, e, e, e, e],
        ],
        3,
    );
    html! {
        <section id="page">
            <BoardComponent board={board}/>
            <div class={classes!("choices")}>
                <CardComponent card={card1}/>
                <CardComponent card={card2}/>
                <CardComponent card={card3}/>
                <CardComponent card={card4}/>
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
                    spaces.iter().flatten().map(|s| view_board_space(s)).collect::<Html>()
                }
            </div>
        </div>
    }
}

fn view_board_space(space: &BoardSpace) -> Html {
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

#[function_component(CardComponent)]
fn card(props: &CardProps) -> Html {
    let card = &props.card;
    html! {
        <div class={classes!("card")}>
            <div>{card.name()}</div>
            <div class={classes!("card-grid")}>
                {
                    card.spaces().iter().flatten().map(|s| view_card_space(s)).collect::<Html>()
                }
            </div>
            <div>{format!("Priority: {}", card.priority())}</div>
            <div>{format!("Special cost: {}", card.special())}</div>
        </div>
    }
}

fn view_card_space(space: &CardSpace) -> Html {
    html! {
        <div class={classes!(match space {
            Some(InkSpace::Normal) => "normal",
            Some(InkSpace::Special) => "special",
            None => "empty"
        })}></div>
    }
}
