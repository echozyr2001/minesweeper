use dioxus::prelude::*;
use std::collections::HashSet;

const MINESWEEPER_CSS: Asset = asset!("/assets/styling/minesweeper.css");

// 游戏难度级别
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Beginner,                    // 9x9, 10 mines
    Intermediate,                // 16x16, 40 mines
    Expert,                      // 16x30, 99 mines
    Custom(usize, usize, usize), // 自定义 (rows, cols, mines)
}

impl Difficulty {
    // 获取行数、列数和雷数
    pub fn dimensions(&self) -> (usize, usize, usize) {
        match self {
            Difficulty::Beginner => (9, 9, 10),
            Difficulty::Intermediate => (16, 16, 40),
            Difficulty::Expert => (16, 30, 99),
            Difficulty::Custom(rows, cols, mines) => (*rows, *cols, *mines),
        }
    }
}

// 游戏状态
#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
    NotStarted,
    Playing,
    Won,
    Lost,
}

// 单元格状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

// 游戏状态
#[derive(Debug, Clone)]
pub struct GameState {
    pub difficulty: Difficulty,
    pub status: GameStatus,
    pub board: Vec<Vec<u8>>, // 0 表示无雷，1-8 表示周围雷数，9 表示雷
    pub cell_states: Vec<Vec<CellState>>,
    pub mines_positions: HashSet<(usize, usize)>,
    pub flags_count: usize,
    pub elapsed_seconds: u32,
}

impl GameState {
    // 创建新游戏
    pub fn new(difficulty: Difficulty) -> Self {
        let (rows, cols, _) = difficulty.dimensions();

        Self {
            difficulty,
            status: GameStatus::NotStarted,
            board: vec![vec![0; cols]; rows],
            cell_states: vec![vec![CellState::Hidden; cols]; rows],
            mines_positions: HashSet::new(),
            flags_count: 0,
            elapsed_seconds: 0,
        }
    }

    // 初始化游戏板（第一次点击后）
    pub fn initialize(&mut self, first_row: usize, first_col: usize) {
        let (rows, cols, mines) = self.difficulty.dimensions();

        // 清空之前的状态
        self.board = vec![vec![0; cols]; rows];
        self.mines_positions.clear();

        // 随机放置地雷，确保第一次点击的位置及其周围没有地雷
        let mut safe_positions = HashSet::new();
        for r in first_row.saturating_sub(1)..=(first_row + 1).min(rows - 1) {
            for c in first_col.saturating_sub(1)..=(first_col + 1).min(cols - 1) {
                safe_positions.insert((r, c));
            }
        }

        let mut mines_placed = 0;
        while mines_placed < mines {
            let row = fastrand::usize(..rows);
            let col = fastrand::usize(..cols);

            if !safe_positions.contains(&(row, col)) && !self.mines_positions.contains(&(row, col))
            {
                self.mines_positions.insert((row, col));
                self.board[row][col] = 9; // 9 表示雷
                mines_placed += 1;
            }
        }

        // 计算每个格子周围的雷数
        for row in 0..rows {
            for col in 0..cols {
                if self.board[row][col] == 9 {
                    continue; // 跳过雷
                }

                let mut count = 0;
                for r in row.saturating_sub(1)..=(row + 1).min(rows - 1) {
                    for c in col.saturating_sub(1)..=(col + 1).min(cols - 1) {
                        if self.board[r][c] == 9 {
                            count += 1;
                        }
                    }
                }
                self.board[row][col] = count;
            }
        }

        self.status = GameStatus::Playing;
    }

    // 揭示格子
    pub fn reveal(&mut self, row: usize, col: usize) -> bool {
        let (rows, cols, _) = self.difficulty.dimensions();

        // 检查是否是第一次点击
        if self.status == GameStatus::NotStarted {
            self.initialize(row, col);
        }

        // 如果游戏已经结束或格子已经揭示或已标记，则不做任何操作
        if self.status != GameStatus::Playing || self.cell_states[row][col] != CellState::Hidden {
            return false;
        }

        // 揭示当前格子
        self.cell_states[row][col] = CellState::Revealed;

        // 如果点到雷，游戏结束
        if self.board[row][col] == 9 {
            self.status = GameStatus::Lost;
            return true;
        }

        // 如果是空格子（周围没有雷），自动揭示周围的格子
        if self.board[row][col] == 0 {
            for r in row.saturating_sub(1)..=(row + 1).min(rows - 1) {
                for c in col.saturating_sub(1)..=(col + 1).min(cols - 1) {
                    if (r != row || c != col) && self.cell_states[r][c] == CellState::Hidden {
                        self.reveal(r, c);
                    }
                }
            }
        }

        // 检查是否获胜
        self.check_win();

        true
    }

    // 标记或取消标记格子
    pub fn toggle_flag(&mut self, row: usize, col: usize) -> bool {
        // 如果游戏已经结束或格子已经揭示，则不做任何操作
        if (self.status != GameStatus::Playing && self.status != GameStatus::NotStarted)
            || self.cell_states[row][col] == CellState::Revealed
        {
            return false;
        }

        match self.cell_states[row][col] {
            CellState::Hidden => {
                self.cell_states[row][col] = CellState::Flagged;
                self.flags_count += 1;
            }
            CellState::Flagged => {
                self.cell_states[row][col] = CellState::Hidden;
                self.flags_count -= 1;
            }
            _ => {}
        }

        true
    }

    // 检查是否获胜
    fn check_win(&mut self) {
        let (rows, cols, mines) = self.difficulty.dimensions();
        let mut hidden_count = 0;

        for row in 0..rows {
            for col in 0..cols {
                if self.cell_states[row][col] == CellState::Hidden
                    || self.cell_states[row][col] == CellState::Flagged
                {
                    hidden_count += 1;
                }
            }
        }

        // 如果隐藏的格子数等于雷数，则获胜
        if hidden_count == mines {
            self.status = GameStatus::Won;
        }
    }

    // 重置游戏
    pub fn reset(&mut self, difficulty: Option<Difficulty>) {
        let difficulty = difficulty.unwrap_or(self.difficulty);
        let (rows, cols, _) = difficulty.dimensions();

        self.difficulty = difficulty;
        self.status = GameStatus::NotStarted;
        self.board = vec![vec![0; cols]; rows];
        self.cell_states = vec![vec![CellState::Hidden; cols]; rows];
        self.mines_positions.clear();
        self.flags_count = 0;
        self.elapsed_seconds = 0;
    }

    // 获取剩余雷数
    pub fn remaining_mines(&self) -> isize {
        let (_, _, mines) = self.difficulty.dimensions();
        mines as isize - self.flags_count as isize
    }
}

// 主扫雷游戏组件
#[component]
pub fn Minesweeper() -> Element {
    let mut game_state = use_signal(|| GameState::new(Difficulty::Beginner));
    let mut show_menu = use_signal(|| false);

    // 格式化时间显示
    let format_time = |seconds: u32| -> String {
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    };

    // 获取表情图标
    let face_icon = match game_state.read().status {
        GameStatus::Won => "😎",
        GameStatus::Lost => "😵",
        _ => "🙂",
    };

    // 获取当前难度
    let current_difficulty = game_state.read().difficulty;

    // 获取游戏板尺寸
    let (rows, cols, _) = current_difficulty.dimensions();

    // 处理菜单显示
    let toggle_menu = move |_| {
        let current = *show_menu.read();
        show_menu.set(!current);
    };

    // 重置游戏
    let reset_game = move |_| {
        game_state.write().reset(None);
    };

    // 选择难度
    let mut select_difficulty = move |difficulty: Difficulty| {
        game_state.write().reset(Some(difficulty));
        show_menu.set(false);
    };

    // 处理单元格点击
    let mut handle_cell_click = move |row: usize, col: usize| {
        game_state.write().reveal(row, col);
    };

    // 处理单元格右键点击
    let mut handle_cell_right_click = move |row: usize, col: usize, e: MouseEvent| {
        e.prevent_default();
        game_state.write().toggle_flag(row, col);
    };

    rsx! {
      div { class: "minesweeper-container",
        // 控制面板
        div { class: "control-panel",
          div { class: "mines-counter", "💣 {game_state.read().remaining_mines()}" }

          div { class: "game-controls",
            button { class: "reset-button", onclick: reset_game, "{face_icon}" }

            div { class: "difficulty-selector",
              button {
                class: "difficulty-button",
                onclick: toggle_menu,
                "难度 ▼"
              }

              div { class: if (*show_menu)() { "difficulty-menu show" } else { "difficulty-menu" },
                button {
                  class: if current_difficulty == Difficulty::Beginner { "active" } else { "" },
                  onclick: move |_| select_difficulty(Difficulty::Beginner),
                  "初级"
                }
                button {
                  class: if current_difficulty == Difficulty::Intermediate { "active" } else { "" },
                  onclick: move |_| select_difficulty(Difficulty::Intermediate),
                  "中级"
                }
                button {
                  class: if current_difficulty == Difficulty::Expert { "active" } else { "" },
                  onclick: move |_| select_difficulty(Difficulty::Expert),
                  "高级"
                }
              }
            }
          }

          div { class: "timer", "⏱️ {format_time(game_state.read().elapsed_seconds)}" }
        }

        // 游戏板
        div {
          class: "game-board",
          style: "grid-template-columns: repeat({cols}, 1fr);",

          for row in 0..rows {
            for col in 0..cols {
              {
                  let value = game_state.read().board[row][col];
                  let state = game_state.read().cell_states[row][col];
                  let game_status = game_state.read().status.clone();
                  let is_mine = value == 9;
                  let number_class = match value {
                      1 => "one",
                      2 => "two",
                      3 => "three",
                      4 => "four",
                      5 => "five",
                      6 => "six",
                      7 => "seven",
                      8 => "eight",
                      _ => "",
                  };
                  let cell_content = match state {
                      CellState::Revealed => {
                          if is_mine {
                              "💣"
                          } else if value == 0 {
                              ""
                          } else {
                              &value.to_string()
                          }
                      }
                      CellState::Flagged => "🚩",
                      CellState::Hidden => "",
                  };
                  let cell_class = match state {
                      CellState::Revealed => {
                          if is_mine && game_status == GameStatus::Lost {
                              "cell revealed mine exploded".to_string()
                          } else if is_mine {
                              "cell revealed mine".to_string()
                          } else if number_class.is_empty() {
                              "cell revealed".to_string()
                          } else {
                              format!("cell revealed {}", number_class)
                          }
                      }
                      CellState::Flagged => {
                          if game_status == GameStatus::Lost && !is_mine {
                              "cell flagged wrong".to_string()
                          } else {
                              "cell flagged".to_string()
                          }
                      }
                      CellState::Hidden => "cell hidden".to_string(),
                  };
                  let row_copy = row;
                  let col_copy = col;
                  rsx! {
                    document::Link { rel: "stylesheet", href: MINESWEEPER_CSS }
                    div {
                      class: "{cell_class}",
                      onclick: move |_| handle_cell_click(row_copy, col_copy),
                      oncontextmenu: move |e| handle_cell_right_click(row_copy, col_copy, e),
                      "{cell_content}"
                    }
                  }
              }
            }
          }
        }
      }
    }
}
