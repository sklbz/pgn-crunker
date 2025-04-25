use chess::bitboard::BitBoardGetter;
use chess::board::Board;
use chess::legal_moves::is_move_possible::is_possible;
use chess::legal_moves::misc::{Color, Square, Type};
use chess::utils::{square_to_string, string_to_square};

pub struct PgnProcessor {
    board: Board,
    current_turn: Color,
}

impl PgnProcessor {
    pub fn new() -> Self {
        PgnProcessor {
            board: Board::init(),
            current_turn: Color::White,
        }
    }

    pub fn reset(&mut self) {
        self.board = Board::init();
        self.current_turn = Color::White;
    }

    fn process_move(&mut self, move_str: &str) -> Option<String> {
        // Handle castling
        if move_str == "O-O" || move_str == "O-O-O" {
            self.board.castle(move_str, &self.current_turn);
            self.current_turn = !self.current_turn;
            return Some(move_str.to_string());
        }

        // Remove check/checkmate symbols
        let cleaned_move = move_str.trim_end_matches('+').trim_end_matches('#');

        // Parse the move
        if let Some((start, end)) = self.parse_move(cleaned_move) {
            let move_tuple = (start, end);
            if is_possible(&self.board, &move_tuple) {
                let result = format!("{}{}", square_to_string(start), square_to_string(end));

                // Update board state
                self.board.play_move(&move_tuple);
                self.current_turn = !self.current_turn;

                return Some(result);
            }
        }

        panic!("Invalid move: {}", move_str);
    }

    fn parse_move(&self, move_str: &str) -> Option<(Square, Square)> {
        // Handle pawn moves (e.g., e4, exd5, e8=Q)
        if move_str.chars().next()?.is_lowercase() {
            return self.parse_pawn_move(move_str);
        }

        // Handle piece moves (e.g., Nf3, Raxa1, Qh4e1)
        if let Some(piece_type) = Self::get_piece_type(move_str.chars().next()?) {
            return self.parse_piece_move(move_str, piece_type);
        }

        panic!("Invalid piece type");
    }

    fn parse_pawn_move(&self, move_str: &str) -> Option<(Square, Square)> {
        let chars: Vec<char> = move_str.chars().collect();
        let mut idx = 0;

        // Check for capture (e.g., exd5)
        let (file, is_capture) = if chars.len() > 1 && chars[1] == 'x' {
            (Some(chars[0]), true)
        } else {
            (None, false)
        };

        if is_capture {
            idx += 2; // Skip capture notation (e.g., 'x')
        }

        // Parse target square
        if chars.len() - idx < 2 {
            return None;
        }

        let target_str: String = chars[idx..].iter().take(2).collect();
        let target_square = string_to_square(&target_str);
        idx += 2;

        // Check for promotion (e.g., e8=Q)
        if idx < chars.len() && chars[idx] == '=' {
            todo!("Promotion");
        }

        // Find the pawn that can make this move
        let pawns = self.board.get_bitboard(&self.current_turn, &Type::Pawn);
        let mut possible_starts = vec![];

        for start_square in pawns.get_occupied_squares() {
            // Check file if specified (for captures or disambiguation)
            if let Some(file) = file {
                let start_file = start_square % 8;
                let expected_file = file as u8 - b'a';
                if start_file != expected_file {
                    continue;
                }
            }

            let move_tuple = (start_square, target_square);
            if is_possible(&self.board, &move_tuple) {
                possible_starts.push(start_square);
            }
        }

        if possible_starts.len() == 1 {
            return Some((possible_starts[0], target_square));
        }

        None
    }

    fn parse_piece_move(&self, move_str: &str, piece_type: Type) -> Option<(Square, Square)> {
        let chars: Vec<char> = move_str.chars().collect();
        let mut idx = 1; // Skip piece character

        // Check for disambiguation (e.g., Nbd2, R1a3, Qh4e1)
        let mut file_disambig = None;
        let mut rank_disambig = None;

        // Check for file disambiguation (e.g., Nbd2)
        if idx + 2 < chars.len() && chars[idx].is_lowercase() && chars[idx] != 'x' {
            file_disambig = Some(chars[idx]);
            idx += 1;
        }
        // Check for rank disambiguation (e.g., N1d2)
        if idx + 2 < chars.len() && chars[idx].is_ascii_digit() {
            rank_disambig = Some(chars[idx]);
            idx += 1;
        }

        // Check for capture (e.g., Nxd2)
        if chars[idx] == 'x' {
            idx += 1;
        }

        if idx + 2 > chars.len() {
            return None;
        }

        let target_str: String = chars[idx..].iter().take(2).collect();
        let target_square = string_to_square(&target_str);

        // Find the piece that can make this move
        let pieces = self.board.get_bitboard(&self.current_turn, &piece_type);

        let mut possible_starts = vec![];

        for start_square in pieces.get_occupied_squares() {
            // Check file disambiguation if specified
            if let Some(file) = file_disambig {
                let start_file = start_square % 8;
                let expected_file = file as u8 - b'a';
                if start_file != expected_file {
                    continue;
                }
            }

            // Check rank disambiguation if specified
            if let Some(rank) = rank_disambig {
                let start_rank = start_square / 8;
                let expected_rank = rank as u8 - b'1';
                if start_rank != expected_rank {
                    continue;
                }
            }

            let move_tuple = (start_square, target_square);
            if is_possible(&self.board, &move_tuple) {
                possible_starts.push(start_square);
            }
        }

        if possible_starts.len() == 1 {
            return Some((possible_starts[0], target_square));
        }

        panic!(
            "Ambiguous move: {}\n target: {}\n possible_starts: {:?}",
            move_str,
            target_str,
            possible_starts
                .iter()
                .map(|s| square_to_string(*s))
                .collect::<Vec<String>>(),
        );
    }

    fn get_piece_type(c: char) -> Option<Type> {
        match c {
            'N' => Some(Type::Knight),
            'B' => Some(Type::Bishop),
            'R' => Some(Type::Rook),
            'Q' => Some(Type::Queen),
            'K' => Some(Type::King),
            _ => {
                println!("Invalid piece type: {}", c);
                None
            }
        }
    }

    pub fn process_pgn(&mut self, pgn: &str) -> Vec<String> {
        let cleaned_pgn = pgn
            .replace("\n", " ")
            .replace("+", "")
            .replace("#", "")
            .replace("1/2-1/2", "")
            .replace("1-0", "")
            .replace("0-1", "")
            .split("1.")
            .skip(1)
            .collect::<String>();

        let mut result = Vec::new();

        for token in cleaned_pgn.split_whitespace() {
            // Skip move numbers and game headers
            if token.ends_with('.') || token.starts_with('[') {
                continue;
            }

            if let Some(processed_move) = self.process_move(token) {
                result.push(processed_move);
            } else {
                eprintln!("Warning: Could not process move '{}'", token);
            }
        }

        result
    }
}
