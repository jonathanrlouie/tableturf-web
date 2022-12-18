use yew::prelude::*;
use common::{CARD_WIDTH, Deck, Hand, HandIndex, Board, BoardSpace, Card, CardSpace, InkSpace, PlayerNum, DeckRng, GameState};
use std::collections::HashSet;

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: Board,
    pub handidx: HandIndex,
    pub selectedcard: Card,
    pub onclick: Callback<(usize, usize)>,
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card: Card,
    pub onclick: Callback<HandIndex>,
    pub handidx: HandIndex,
    pub selected: bool,
}

#[derive(Copy, Clone)]
enum Phase {
    Redraw,
    WaitForRedraw,
    Battle,
    WaitForBattleInput,
    GameEnd
}

#[derive(Clone)]
struct BattleState {
    hand: Hand,
    deck: Deck,
    hand_idx: HandIndex,
    board: Board,
    phase: Phase
}

#[derive(Clone)]
struct CursorState {
    cursor: HashSet<(usize, usize)>
}

#[function_component(Battle)]
pub fn battle() -> Html {
    let state = use_state(|| {
        let game_state = GameState::<DeckRng>::default();
        let player = game_state.player(PlayerNum::P1);
        
        BattleState {
            hand: player.hand().clone(),
            deck: player.deck().clone(),
            hand_idx: HandIndex::H1,
            board: game_state.board().clone(),
            phase: Phase::Battle,
        }
    });
    let onclick_card = {
        let state = state.clone();
        Callback::from(move |hand_idx| {
            state.set(BattleState {
                hand_idx,
                ..(*state).clone()
            });
        })
    };
    let onclick_space = {
        let state = state.clone();
        Callback::from(move |(x, y)| {
            /*
            state.sender.send(RawInput {
                hand_idx: props.handidx,
                action: Action::Place(RawPlacement {
                    x: x,
                    y: y,
                    special_activated: false,
                    rotation: Rotation::Zero
                })
            }).unwrap();
            */
        })
    };
    let board = state.board.clone();
    let hand = state.hand.clone();
    let deck = state.deck.clone();
    let (card1, _) = deck.index(hand[HandIndex::H1]);
    let card1 = card1.clone();
    let (card2, _) = deck.index(hand[HandIndex::H2]);
    let card2 = card2.clone();
    let (card3, _) = deck.index(hand[HandIndex::H3]);
    let card3 = card3.clone();
    let (card4, _) = deck.index(hand[HandIndex::H4]);
    let card4 = card4.clone();
    let (selected_card, _) = deck.index(hand[state.hand_idx]);
    let selected_card = selected_card.clone();
    html! {
        <section id="page">
            <BoardComponent
                board={board}
                handidx={state.hand_idx}
                selectedcard={selected_card}
                onclick={onclick_space}
                />
            <div class={classes!("choices")}>
                <CardComponent
                    card={card1}
                    onclick={onclick_card.clone()}
                    handidx={HandIndex::H1}
                    selected={state.hand_idx == HandIndex::H1}/>
                <CardComponent
                    card={card2}
                    onclick={onclick_card.clone()}
                    handidx={HandIndex::H2}
                    selected={state.hand_idx == HandIndex::H2}/>
                <CardComponent
                    card={card3}
                    onclick={onclick_card.clone()}
                    handidx={HandIndex::H3}
                    selected={state.hand_idx == HandIndex::H3}/>
                <CardComponent
                    card={card4}
                    onclick={onclick_card.clone()}
                    handidx={HandIndex::H4}
                    selected={state.hand_idx == HandIndex::H4}/>
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
    let state = use_state(|| CursorState { cursor: HashSet::new() });
    let width = props.board.width();
    let height = props.board.height();
    let spaces = props.board.spaces();
    let onmouseover_space = {
        let state = state.clone();
        Callback::from(move |(x, y, card): (usize, usize, Card)| {
            let mut cursor = HashSet::new();
            let spaces = card.spaces();
            let ink_spaces = spaces.iter()
                .flatten()
                .enumerate()
                .filter(|(_, s)| s.is_some());
            for (idx, _) in ink_spaces {
                let card_x = idx % CARD_WIDTH;
                let card_y = idx / CARD_WIDTH;
                match (usize::checked_sub(x + card_x, 4), usize::checked_sub(y + card_y, 4)) {
                    (Some(x), Some(y)) => {
                        cursor.insert((x, y));
                    }
                    _ => ()
                }
            };
            state.set(CursorState {
                cursor 
            });
        })
    };
    html! {
        <div class={classes!("board")}>
            <div 
                class={classes!("board-grid")}
                style={format!("display: grid; grid-template-rows: repeat({}, 1fr); grid-template-columns: repeat({}, 1fr)", height, width)}>
                {
                    spaces.iter().enumerate().map(|(idx, s)| {
                        let x = idx % width;
                        let y = idx / width;
                        board_space(
                            (x, y),
                            &(*state).cursor,
                            s,
                            props.onclick.clone(),
                            onmouseover_space.clone(),
                            props.selectedcard.clone()
                        )
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}

fn board_space(
    position: (usize, usize),
    cursor: &HashSet<(usize, usize)>,
    space: &BoardSpace,
    onclick_space: Callback<(usize, usize)>,
    onmouseover_space: Callback<(usize, usize, Card)>,
    selected_card: Card,
) -> Html {
    let onclick = Callback::from(move |_| {
        onclick_space.emit(position);
    });
    let onmouseover = Callback::from(move |_| {
        onmouseover_space.emit((position.0, position.1, selected_card.clone()));
    });
    let mut class = match space {
        BoardSpace::Empty => classes!("bordered", "empty"),
        BoardSpace::Ink { player_num } => {
            classes!(get_player_num_class(player_num), "ink", "bordered")
        }
        BoardSpace::Special { player_num, is_activated } => {
            get_special_classes(player_num, *is_activated)
        }
        BoardSpace::Wall => classes!("wall", "bordered"),
        BoardSpace::OutOfBounds => classes!("oob"),
    };
    class.extend(classes!("board-space"));
    html! {
        <div class={class} {onclick} {onmouseover}>
            {
                if cursor.contains(&position) {
                    html! {
                        <div class={classes!("board-cursor")}></div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
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
        classes!(get_player_num_class(player_num), "special", "activated", "bordered")
    } else {
        classes!(get_player_num_class(player_num), "special", "bordered")
    }
} 

#[function_component(CardComponent)]
fn card(props: &CardProps) -> Html {
    let card = &props.card;
    let callback = props.onclick.clone();
    let hand_idx = props.handidx;
    let onclick = Callback::from(move |_| {
        callback.emit(hand_idx);
    });
    html! {
        <button class={if props.selected { classes!("card", "selected") } else { classes!("card") }} {onclick}>
            <div>{card.name()}</div>
            <div class={classes!("card-grid")}>
                {
                    card.spaces().iter().flatten().map(|s| card_space(s)).collect::<Html>()
                }
            </div>
            <div>{format!("Priority: {}", card.priority())}</div>
            <div>{format!("Special cost: {}", card.special())}</div>
        </button>
    }
}

fn card_space(space: &CardSpace) -> Html {
    html! {
        <div class={classes!(match space {
            Some(InkSpace::Normal) => "normal",
            Some(InkSpace::Special) => "special",
            None => "empty"
        })}></div>
    }
}
