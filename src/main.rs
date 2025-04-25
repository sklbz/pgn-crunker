use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use chess::board::Board;
use chess::legal_moves::misc::{Color, Move, Square, Type};
use chess::utils::{square_to_string, string_to_square};

struct PgnProcessor {
    board: Board,
    current_turn: Color,
}

impl PgnProcessor {
    fn new() -> Self {
        PgnProcessor {
            board: Board::init(),
            current_turn: Color::White,
        }
    }

    fn reset(&mut self) {
        self.board = Board::init();
        self.current_turn = Color::White;
    }

    fn process_move(&mut self, move_str: &str) -> Option<String> {
        // Handle castling
        if move_str == "O-O" || move_str == "O-O-O" {
            self.current_turn = !self.current_turn;
            return Some(move_str.to_string());
        }

        // Remove check/checkmate symbols
        let cleaned_move = move_str.trim_end_matches('+').trim_end_matches('#');

        // Parse the move
        if let Some((start, end)) = self.parse_move(cleaned_move) {
            let move_tuple = (start, end);
            if chess::legal_moves::is_move_possible::is_possible(&self.board, &move_tuple) {
                let result = format!("{}{}", square_to_string(start), square_to_string(end));

                // Update board state
                self.board.play_move(&move_tuple);
                self.current_turn = !self.current_turn;

                return Some(result);
            }
        }

        None
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

        None
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
            // Promotion - we don't need to handle this for our format
            idx += 2;
        }

        // Find the pawn that can make this move
        let pawns = self.board.get_bitboard(&self.current_turn, &Type::Pawn);
        let mut possible_starts = vec![];

        for start_square in pawns.get_occupied_squares() {
            // Check file if specified (for captures or disambiguation)
            if let Some(file) = file {
                let start_file = (start_square % 8) as u8;
                let expected_file = (file as u8 - b'a') as u8;
                if start_file != expected_file {
                    continue;
                }
            }

            let move_tuple = (start_square, target_square);
            if chess::legal_moves::is_move_possible::is_possible(&self.board, &move_tuple) {
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
        if idx < chars.len() && chars[idx].is_lowercase() {
            file_disambig = Some(chars[idx]);
            idx += 1;
        }
        // Check for rank disambiguation (e.g., N1d2)
        else if idx < chars.len() && chars[idx].is_digit(10) {
            rank_disambig = Some(chars[idx]);
            idx += 1;
        }

        // Check for capture (e.g., Nxd2)
        if idx < chars.len() && chars[idx] == 'x' {
            idx += 1;
        }

        // Parse target square
        if chars.len() - idx < 2 {
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
                let start_file = (start_square % 8) as u8;
                let expected_file = (file as u8 - b'a') as u8;
                if start_file != expected_file {
                    continue;
                }
            }

            // Check rank disambiguation if specified
            if let Some(rank) = rank_disambig {
                let start_rank = (start_square / 8) as u8;
                let expected_rank = (rank as u8 - b'1') as u8;
                if start_rank != expected_rank {
                    continue;
                }
            }

            let move_tuple = (start_square, target_square);
            if chess::legal_moves::is_move_possible::is_possible(&self.board, &move_tuple) {
                possible_starts.push(start_square);
            }
        }

        if possible_starts.len() == 1 {
            return Some((possible_starts[0], target_square));
        }

        None
    }

    fn get_piece_type(c: char) -> Option<Type> {
        match c {
            'N' => Some(Type::Knight),
            'B' => Some(Type::Bishop),
            'R' => Some(Type::Rook),
            'Q' => Some(Type::Queen),
            'K' => Some(Type::King),
            _ => None,
        }
    }

    fn process_pgn(&mut self, pgn: &str) -> Vec<String> {
        let mut result = Vec::new();

        for token in pgn.split_whitespace() {
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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file
        let path = Path::new(&args[1]);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut pgn = String::new();
        for line in reader.lines() {
            pgn.push_str(&line?);
            pgn.push(' ');
        }
        pgn
    } else {
        // Read from stdin
        println!("Enter PGN (press Ctrl+D when done):");
        let stdin = io::stdin();
        let mut pgn = String::new();
        for line in stdin.lock().lines() {
            pgn.push_str(&line?);
            pgn.push(' ');
        }
        pgn
    };

    let mut processor = PgnProcessor::new();
    let processed_moves = processor.process_pgn(&input);

    println!("Processed moves:");
    for (i, mv) in processed_moves.iter().enumerate() {
        if i % 2 == 0 {
            print!("{}. ", i / 2 + 1);
        }
        print!("{} ", mv);
        if i % 2 == 1 {
            println!();
        }
    }
    if processed_moves.len() % 2 == 1 {
        println!();
    }

    // Write output to file if desired
    if args.len() > 2 {
        let mut output_file = File::create(&args[2])?;
        writeln!(output_file, "{}", processed_moves.join(" "))?;
        println!("Output written to {}", &args[2]);
    }

    Ok(())
}
