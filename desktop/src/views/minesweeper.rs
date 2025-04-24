use dioxus::prelude::*;
use ui::Minesweeper;

#[component]
pub fn MinesweeperView() -> Element {
    rsx! {
      div { class: "minesweeper-view", Minesweeper {} }
    }
}
