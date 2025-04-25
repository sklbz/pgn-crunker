#[test]
fn test_pgn() {
    use crate::PgnProcessor;
    let game = "1. e4 e5 2. f4 d5 3. Nf3 Bg4 4. Be2 dxe4 5. Nxe5 Bxe2 6. Qxe2 Bd6 7. Qxe4 Nd7 8.
d4 Ngf6 9. Qe2 O-O 10. O-O Re8 11. Nc3 Qe7 12. Re1 Qe6 13. b3 Bb4 14. Bb2 Nd5
15. Qh5 Nxc3 16. f5 Qf6 17. Bc1 Nd5 18. Bg5 Qxf5 19. Rf1 Qe6 20. Ng4 g6 21. Nh6+
Kg7 22. Bf6+ N5xf6 23. Qg5 Nh5 24. Rxf7+ 1-0";

    let mut processor = PgnProcessor::new();
    let processed_moves = processor.process_pgn(game);

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
}
