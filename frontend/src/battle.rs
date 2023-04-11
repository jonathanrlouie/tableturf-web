use yew::prelude::*;
use common::{CARD_WIDTH, Deck, Hand, HandIndex, Board, BoardSpace, Card, CardSpace, InkSpace, PlayerNum, DeckRng, GameState, RawInput, Rotation, Action, RawPlacement};
use std::collections::HashSet;
use std::rc::Rc;
use crate::User;
use futures::channel::mpsc::Sender;
use crate::worker::WebSocketWorker;
use common::messages::Response;
use yew_agent::{Bridge, Bridged};
use crate::ws;

const CURSOR_OFFSET: usize = 4;

pub enum Message {
    ClickCard(HandIndex),
    ClickSpace(usize, usize),
    WorkerMsg(String)
}

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

#[derive(Clone)]
enum Phase {
    WaitingForGameStart,
    Redraw,
    WaitForBattleStart,
    Battle,
    WaitForOpponentInput,
    GameEnd
}

#[derive(Clone)]
struct BattleState {
    board: Board,
    hand_idx: HandIndex,
    hand: Hand,
    deck: Deck,
}

#[derive(Clone)]
struct CursorState {
    cursor: HashSet<(usize, usize)>
}

pub struct Battle {
    ws_sender: Sender<String>,
    phase: Phase,
    state: BattleState,
    worker: Box<dyn Bridge<WebSocketWorker>>
}

impl Component for Battle {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        // forward message returned from worker to Component's update method
        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Self::Message::WorkerMsg(msg))
        };
        let worker = WebSocketWorker::bridge(Rc::new(cb));
        let mut ws_sender = ws::connect(user.user_id.borrow().clone());
        ws_sender.try_send("join".to_string()).unwrap();

        // TODO: these vars are temporary. delete later.
        let game_state = GameState::<DeckRng>::default();
        let player = game_state.player(PlayerNum::P1);

        Self {
            ws_sender,
            //phase: Phase::WaitingForGameStart,
            phase: Phase::Battle,
            state: BattleState {
                board: game_state.board().clone(),
                hand_idx: HandIndex::H1,
                hand: player.hand().clone(),
                deck: player.deck().clone(),
            },
            worker,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::WorkerMsg(response) => {
                let response: Response = serde_json::from_str(&response).unwrap();
                match response {
                    Response::Redraw { player } => {}
                    Response::GameState { board, player } => {}
                    Response::GameEnd { outcome } => {}
                }
            }
            Message::ClickCard(hand_idx) => {
                self.state = BattleState {
                    hand_idx,
                    ..self.state.clone()
                };
            }
            Message::ClickSpace(x, y) => {
                let input = RawInput {
                    hand_idx: self.state.hand_idx,
                    action: Action::Place(RawPlacement {
                        x,
                        y,
                        special_activated: false,
                        rotation: Rotation::Zero,
                    }),
                };
                self.ws_sender.try_send(serde_json::to_string(&input).unwrap()).unwrap();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.phase {
            Phase::Battle => self.view_battle(ctx),
            _ => html! {}
        }
    }
}
    
impl Battle {
    fn view_battle(&self, ctx: &Context<Self>) -> Html {
        let onclick_space = ctx.link().callback(|(x, y)| Message::ClickSpace(x, y));
        let onclick_card = ctx.link().callback(|hand_idx| Message::ClickCard(hand_idx));
        let board = self.state.board.clone();
        let hand = self.state.hand.clone();
        let deck = self.state.deck.clone();
        let (card1, _) = deck.index(hand[HandIndex::H1]);
        let card1 = card1.clone();
        let (card2, _) = deck.index(hand[HandIndex::H2]);
        let card2 = card2.clone();
        let (card3, _) = deck.index(hand[HandIndex::H3]);
        let card3 = card3.clone();
        let (card4, _) = deck.index(hand[HandIndex::H4]);
        let card4 = card4.clone();
        let (selected_card, _) = deck.index(hand[self.state.hand_idx]);
        let selected_card = selected_card.clone();
        html! {
            <section id="page">
                <BoardComponent
                    board={board}
                    handidx={self.state.hand_idx}
                    selectedcard={selected_card}
                    onclick={onclick_space}/>
                <div class={classes!("choices")}>
                    <CardComponent
                        card={card1}
                        onclick={onclick_card.clone()}
                        handidx={HandIndex::H1}
                        selected={self.state.hand_idx == HandIndex::H1}/>
                    <CardComponent
                        card={card2}
                        onclick={onclick_card.clone()}
                        handidx={HandIndex::H2}
                        selected={self.state.hand_idx == HandIndex::H2}/>
                    <CardComponent
                        card={card3}
                        onclick={onclick_card.clone()}
                        handidx={HandIndex::H3}
                        selected={self.state.hand_idx == HandIndex::H3}/>
                    <CardComponent
                        card={card4}
                        onclick={onclick_card.clone()}
                        handidx={HandIndex::H4}
                        selected={self.state.hand_idx == HandIndex::H4}/>
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
                match (usize::checked_sub(x + card_x, CURSOR_OFFSET), usize::checked_sub(y + card_y, CURSOR_OFFSET)) {
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
        callback.emit(hand_idx)
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
