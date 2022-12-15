use yew::prelude::*;

#[function_component(Battle)]
pub fn battle() -> Html {
    html! {
        <section id="page">
            <div class="board">
                <div class="board-grid">
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                    <div class="board-space"></div>
                </div>
            </div>
            <div class="choices">
                <div class="card">
                    <div>Splattershot</div>
                    <div class="card-grid">
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                    </div>
                    <div>Priority: 6</div>
                    <div>Special cost: 3</div>
                </div>
                <div class="card">
                    <div>Flyfish</div>
                    <div class="card-grid">
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                    </div>
                    <div>Priority: 13</div>
                    <div>Special cost: 5</div>
                </div>
                <div class="card">
                    <div>Splatterscope</div>
                    <div class="card-grid">
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                    </div>
                    <div>Priority: 9</div>
                    <div>Special cost: 3</div>
                </div>
                <div class="card">
                    <div>Splat Bomb</div>
                    <div class="card-grid">
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                        <div class="card-space"></div>
                    </div>
                    <div>Priority: 3</div>
                    <div>Special cost: 1</div>
                </div>
                <button>Pass</button>
                <button>Special</button>
            </div>
            <div class="timer">
                <div>Turns left: 12</div>
                <div>Time remaining: 1:30</div> 
            </div>
            <div class="special-gauge">Special gauge: 0</div>
            <button class="deck">View deck</button>
        </section>
    }
}
