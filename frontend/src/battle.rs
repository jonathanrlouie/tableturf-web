use crate::worker::WebSocketWorker;
use crate::ws;
use crate::User;
use common::messages::{GameEnd, GameState as GameStateMsg};
use common::{
    Action, Board, BoardSpace, Card, CardSpace, Deck, DeckRng, GameState, Grid, Hand, HandIndex,
    InkSpace, Player, PlayerNum, RawInput, RawPlacement, Rotation, ValidInput, CARD_WIDTH,
};
use futures::channel::mpsc::Sender;
use gloo::console::log;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

const CURSOR_OFFSET: usize = 4;

#[derive(Debug, Clone)]
pub enum Message {
    GameInput(GameInput),
    WorkerMsg(String),
}

#[derive(Debug, Clone)]
pub enum GameInput {
    Redraw,
    KeepHand,
    Pass,
    ClickCard(HandIndex),
    RotateCard(Rotation),
    ClickSpace(usize, usize),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::GameInput(GameInput::Redraw) => write!(f, "Redraw"),
            Message::GameInput(GameInput::KeepHand) => write!(f, "KeepHand"),
            Message::GameInput(GameInput::Pass) => write!(f, "Pass"),
            Message::GameInput(GameInput::ClickCard(idx)) => write!(f, "ClickCard: {:?}", idx),
            Message::GameInput(GameInput::RotateCard(rotation)) => {
                write!(f, "RotateCard: {:?}", rotation)
            }
            Message::GameInput(GameInput::ClickSpace(x, y)) => {
                write!(f, "ClickSpace: {:?}, {:?}", x, y)
            }
            Message::WorkerMsg(s) => write!(f, "WorkerMsg: {}", s),
        }
    }
}

// There is a separate phase for searching for an opponent because the battle state doesn't exist yet at that point
#[derive(Clone, Debug)]
enum Phase {
    SearchingForOpponent,
    Battling(BattleState),
}

#[derive(Clone, Debug)]
enum BattlePhase {
    // Phase where player needs to choose to redraw their hand or not
    Redraw,
    // Phase where player waits for opponent to choose if they want to redraw their hand
    WaitingForBattleStart,
    // Phase where player needs to place a card or pass
    Input,
    // Phase where player is waiting for opponent to place a card or pass
    WaitingForOpponentInput,
    // Phase where game is over and player needs to choose to rematch or not
    GameEnd,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::SearchingForOpponent => write!(f, "Redraw"),
            Phase::Battling(state) => write!(f, "Battling with state: {:?}", state),
        }
    }
}

#[derive(Clone, Debug)]
struct BattleState {
    phase: BattlePhase,
    board: Board,
    player: Player,
    hand_idx: HandIndex,
    rotation: Rotation,
    turns_left: u32,
}

#[derive(Clone)]
struct CursorState {
    cursor: HashSet<(usize, usize)>,
}

pub struct Battle {
    ws_sender: Sender<String>,
    phase: Phase,
    worker: Box<dyn Bridge<WebSocketWorker>>,
}

// Processes a response from the backend server
fn process_response(phase: &mut Phase, response: String) {
    match *phase {
        Phase::SearchingForOpponent => {
            log!("Entering Redraw state");
            let game_state: GameStateMsg = serde_json::from_str(&response).unwrap();
            *phase = Phase::Battling(BattleState {
                phase: BattlePhase::Redraw,
                board: game_state.board,
                player: game_state.player,
                hand_idx: HandIndex::H1,
                rotation: Rotation::Zero,
                turns_left: 12,
            });
        }
        Phase::Battling(ref mut state) => process_battle_response(response, state),
    }
}

fn process_battle_response(response: String, state: &mut BattleState) {
    match state.phase {
        BattlePhase::Redraw => {}
        BattlePhase::WaitingForBattleStart => {
            let game_state: GameStateMsg = serde_json::from_str(&response).unwrap();
            state.board = game_state.board;
            state.hand_idx = HandIndex::H1;
            state.player = game_state.player.clone();
            state.phase = BattlePhase::Input;
        }
        BattlePhase::Input => {}
        BattlePhase::WaitingForOpponentInput => {
            if state.turns_left == 0 {
                let game_state: GameEnd = serde_json::from_str(&response).unwrap();
                state.phase = BattlePhase::GameEnd;
            } else {
                let game_state: GameStateMsg = serde_json::from_str(&response).unwrap();
                state.board = game_state.board;
                state.hand_idx = HandIndex::H1;
                state.player = game_state.player.clone();
                state.phase = BattlePhase::Input;
                state.turns_left -= 1;
            }
        }
        BattlePhase::GameEnd => {}
    }
}

// Makes sure the player has placed their ink in a legal position before sending to the server
// Returns the raw input that will be sent to the server
fn validate_placement(x: usize, y: usize, state: &BattleState) -> Option<RawInput> {
    let x = usize::checked_sub(x, CURSOR_OFFSET)?;
    let y = usize::checked_sub(y, CURSOR_OFFSET)?;

    let input = RawInput {
        hand_idx: state.hand_idx,
        action: Action::Place(RawPlacement {
            x,
            y,
            special_activated: false,
            rotation: state.rotation,
        }),
    };

    // Check that we got a valid input, but don't return it, since we want the raw input for easier deserialization
    ValidInput::new(input.clone(), &state.board, &state.player).ok()?;
    Some(input)
}

fn process_input(ws_sender: &mut Sender<String>, input: GameInput, state: &mut BattleState) {
    match input {
        GameInput::Redraw => {
            state.phase = BattlePhase::WaitingForBattleStart;
            ws_sender
                .try_send(serde_json::to_string(&true).unwrap())
                .unwrap();
        }
        GameInput::KeepHand => {
            state.phase = BattlePhase::WaitingForBattleStart;
            ws_sender
                .try_send(serde_json::to_string(&false).unwrap())
                .unwrap();
        }
        GameInput::ClickCard(hand_idx) => {
            state.hand_idx = hand_idx;
        }
        GameInput::RotateCard(rotation) => {
            state.rotation = rotation;
        }
        GameInput::Pass => {
            let input = RawInput {
                hand_idx: state.hand_idx,
                action: Action::Pass,
            };
            ws_sender
                .try_send(serde_json::to_string(&input).unwrap())
                .unwrap();
            state.phase = BattlePhase::WaitingForOpponentInput;
        }
        GameInput::ClickSpace(x, y) => {
            if let Some(input) = validate_placement(x, y, state) {
                ws_sender
                    .try_send(serde_json::to_string(&input).unwrap())
                    .unwrap();
                state.phase = BattlePhase::WaitingForOpponentInput;
            }
        }
    }
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

        Self {
            ws_sender,
            phase: Phase::SearchingForOpponent,
            worker,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log!("Entering update");

        match (msg.clone(), &mut self.phase) {
            (Message::WorkerMsg(response), phase) => process_response(phase, response),
            (Message::GameInput(input), Phase::Battling(ref mut state)) => {
                process_input(&mut self.ws_sender, input, state)
            }
            _ => log!(
                "Invalid message and phase: message: {}, phase: {}",
                msg.to_string(),
                self.phase.to_string()
            ),
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log!("view function entered");
        match &self.phase {
            Phase::SearchingForOpponent => html! { "Searching for opponent..." },
            Phase::Battling(state) => view_battle(ctx, &state),
        }
    }
}

fn view_battle(ctx: &Context<Battle>, state: &BattleState) -> Html {
    match state.phase {
        BattlePhase::Redraw => view_redraw(ctx, state),
        BattlePhase::Input => view_input(ctx, state),
        BattlePhase::WaitingForOpponentInput => html! { "Waiting for opponent input" },
        _ => html! { "non-battle phase" },
    }
}

fn view_redraw(ctx: &Context<Battle>, state: &BattleState) -> Html {
    let onclick_redraw = ctx
        .link()
        .callback(|_| Message::GameInput(GameInput::Redraw));
    let onclick_keep = ctx
        .link()
        .callback(|_| Message::GameInput(GameInput::KeepHand));
    html! {
        <section id="page">
            <button onclick={onclick_redraw}>{"Redraw"}</button>
            <button onclick={onclick_keep}>{"Keep hand"}</button>
        </section>
    }
}

fn view_input(ctx: &Context<Battle>, state: &BattleState) -> Html {
    let onclick_space = ctx
        .link()
        .callback(|(x, y)| Message::GameInput(GameInput::ClickSpace(x, y)));
    let onclick_card = ctx
        .link()
        .callback(|hand_idx| Message::GameInput(GameInput::ClickCard(hand_idx)));
    let onrightclick_space = ctx
        .link()
        .callback(|rotation| Message::GameInput(GameInput::RotateCard(rotation)));
    let onclick_pass = ctx.link().callback(|_| Message::GameInput(GameInput::Pass));
    let board = state.board.clone();
    let player = state.player.clone();
    let special = state.player.special;
    let rotation = state.rotation;
    let player_num = player.player_num();
    let hand = player.hand().clone();
    let deck = player.deck().clone();
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
                rotation={rotation}
                onclick={onclick_space}
                onrightclick={onrightclick_space}
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
                <PassButtonComponent onclick={onclick_pass.clone()}/>
                <button>{"Special"}</button>
            </div>
            <div class={classes!("timer")}>
                <div>{format!("Turns left: {}", state.turns_left)}</div>
                <div>{"Time remaining: 1:30"}</div>
                <div>{format!("Player number: {}", match player_num {
                    PlayerNum::P1 => "1",
                    PlayerNum::P2 => "2",
                })}</div>
            </div>
            <div class={classes!("special-gauge")}>{format!("Special gauge: {}", special)}</div>
            <button class={classes!("deck")}>{"View deck"}</button>
        </section>
    }
}

#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: Board,
    pub handidx: HandIndex,
    pub selectedcard: Card,
    pub rotation: Rotation,
    pub onclick: Callback<(usize, usize)>,
    pub onrightclick: Callback<Rotation>,
}

#[function_component(BoardComponent)]
pub fn board(props: &BoardProps) -> Html {
    let state = use_state(|| CursorState {
        cursor: HashSet::new(),
    });
    let width = props.board.width();
    let height = props.board.height();
    let spaces = props.board.spaces();
    let card = props.selectedcard.clone();
    let rotation = props.rotation;
    let onrightclick_space = props.onrightclick.clone();
    let onrightclick = {
        let state = state.clone();
        Callback::from(move |(x, y): (usize, usize)| {
            let rotation = match rotation {
                Rotation::Zero => Rotation::One,
                Rotation::One => Rotation::Two,
                Rotation::Two => Rotation::Three,
                Rotation::Three => Rotation::Zero,
            };
            onrightclick_space.emit(rotation);
            let cursor = update_cursor(x, y, &card, rotation);
            state.set(CursorState { cursor });
        })
    };
    let card = props.selectedcard.clone();
    let onmouseover_space = {
        let state = state.clone();
        Callback::from(move |(x, y): (usize, usize)| {
            let cursor = update_cursor(x, y, &card, rotation);
            state.set(CursorState { cursor });
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
                            onrightclick.clone(),
                        )
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}

fn update_cursor(x: usize, y: usize, card: &Card, rotation: Rotation) -> HashSet<(usize, usize)> {
    let mut cursor = HashSet::new();
    let spaces = common::rotate_input(card, rotation);
    let ink_spaces = spaces
        .iter()
        .flatten()
        .enumerate()
        .filter(|(_, s)| s.is_some());
    for (idx, _) in ink_spaces {
        let card_x = idx % CARD_WIDTH;
        let card_y = idx / CARD_WIDTH;
        match (
            usize::checked_sub(x + card_x, CURSOR_OFFSET),
            usize::checked_sub(y + card_y, CURSOR_OFFSET),
        ) {
            (Some(x), Some(y)) => {
                cursor.insert((x, y));
            }
            _ => (),
        }
    }
    cursor
}

fn board_space(
    position: (usize, usize),
    cursor: &HashSet<(usize, usize)>,
    space: &BoardSpace,
    onclick_space: Callback<(usize, usize)>,
    onmouseover_space: Callback<(usize, usize)>,
    onrightclick: Callback<(usize, usize)>,
) -> Html {
    let onclick = Callback::from(move |_| {
        onclick_space.emit(position);
    });
    let onmouseover = Callback::from(move |_| {
        onmouseover_space.emit(position);
    });
    let oncontextmenu = Callback::from(move |e: MouseEvent| {
        onrightclick.emit(position);
        e.prevent_default();
    });
    let mut class = match space {
        BoardSpace::Empty => classes!("bordered", "empty"),
        BoardSpace::Ink { player_num } => {
            classes!(get_player_num_class(player_num), "ink", "bordered")
        }
        BoardSpace::Special {
            player_num,
            is_activated,
        } => get_special_classes(player_num, *is_activated),
        BoardSpace::Wall => classes!("wall", "bordered"),
        BoardSpace::OutOfBounds => classes!("oob"),
    };
    class.extend(classes!("board-space"));
    html! {
        <div class={class} {onclick} {onmouseover} {oncontextmenu}>
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
        classes!(
            get_player_num_class(player_num),
            "special",
            "activated",
            "bordered"
        )
    } else {
        classes!(get_player_num_class(player_num), "special", "bordered")
    }
}

#[derive(Properties, PartialEq)]
pub struct PassButtonProps {
    pub onclick: Callback<()>,
}

#[function_component(PassButtonComponent)]
pub fn pass_button(props: &PassButtonProps) -> Html {
    let onclick_pass = props.onclick.clone();
    let onclick = Callback::from(move |_| {
        onclick_pass.emit(());
    });
    html! {
        <button {onclick}>{"Pass"}</button>
    }
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card: Card,
    pub onclick: Callback<HandIndex>,
    pub handidx: HandIndex,
    pub selected: bool,
}

#[function_component(CardComponent)]
fn card(props: &CardProps) -> Html {
    let card = &props.card;
    let callback = props.onclick.clone();
    let hand_idx = props.handidx;
    let onclick = Callback::from(move |_| callback.emit(hand_idx));
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
