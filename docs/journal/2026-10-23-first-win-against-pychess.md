# 2026-10-23: First win against PyChess's engine

## Goal
Keep a record of a milestone game: Stargazer's first win against
PyChess.py, the engine built into the PyChess GUI, at full strength.
(Its first win of all was against Charlie, 2026-10-20-first-game.md.)

## What was done
Charlie played the two engines against each other in PyChess on the
Chromebook at 5 minutes each, no increment, Stargazer with White. The
build was `main` just after the transposition table merged (#19); the
engine still calls itself 0.1.0.

Earlier the same day the build from before the table (#16 only) had lost
to PyChess, with PyChess's strength slider turned down: a queen up at
move 44, it shuffled for dozens of moves, blundered twice once its clock
was nearly empty, and lost on time after 1,445 half-moves. This time
PyChess was at full strength and Stargazer won in 42 moves.

```
[Event "Local Event"]
[Site "Local Site"]
[Date "2026.10.23"]
[Round "1"]
[White "Stargazer 0.1.0"]
[Black "PyChess.py"]
[Result "1-0"]
[PlyCount "83"]
[Termination "normal"]
[WhiteClock "0:00:25.283"]
[BlackClock "0:00:25.156"]
[TimeControl "300+0"]
[ECO "A45"]
[Opening "Trompowsky Attack"]
[Variation "Classical Defense, Big Center Variation"]

1. d4 Nf6 2. Bg5 e6 3. e4 h6 4. Bxf6 Qxf6 5. c3 d6 6. Bd3 g6 7. Ne2 Bg7 8. b4
O-O 9. Nd2 e5 10. b5 Bg4 11. f3 Be6 12. d5 Bd7 13. O-O Qg5 14. f4 exf4 15. Rxf4
f5 16. Nf3 Qe7 17. Ned4 Qe8 18. a4 g5 19. Rxf5 Bxf5 20. Nxf5 Qf7 21. N3d4 Bxd4+
22. cxd4 Kh8 23. Rc1 a6 24. Qc2 axb5 25. axb5 Ra3 26. Bf1 h5 27. Qxc7 Qxc7 28.
Rxc7 Ra1 29. Kf2 Ra2+ 30. Be2 Ra7 31. Bxh5 Ra2+ 32. Kg3 Ra3+ 33. Bf3 Ra4 34.
Rxb7 Rb4 35. Kg4 Ra4 36. Nh6 Rxd4 37. Kxg5 Rb4 38. Kg6 Nd7 39. Rxd7 Rg8+ 40.
Nxg8 Rd4 41. Nf6 Rc4 42. Rh7# 1-0
```

Moments worth a look:
- **19. Rxf5:** a rook given for a bishop to put a knight on f5, in front
  of Black's king. The material came back later, with the c-pawn and
  then the attack.
- **27. Qxc7:** the queens come off as White wins the c-pawn; from there
  White's rook and minor pieces outplay Black's.
- **The finish:** the king walks up to g6, 40. Nxg8 takes the rook that
  has just given check, and 42. Rh7 is mate: the king covers g7 and the
  knight on f6 covers g8.

## Problems and fixes
None in the game. The clock still drains at 5+0: 25 seconds left after
42 moves. A short game hides it; a long one wouldn't (the time rework on
the roadmap).

## How it was verified
The PGN is as exported by PyChess and pasted into the project thread by
Charlie. Replayed with the python-chess library: 83 legal half-moves,
ending in checkmate.

## Impact
None to the code. A record of the day the engine first beat another
engine at full strength, a few hours after the transposition table
(+136 Elo in SPRT) went in.
