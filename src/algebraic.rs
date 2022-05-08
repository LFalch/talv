use std::{fmt::{self, Display}};
use std::str::Chars;
use std::iter::{Iterator};

use crate::location::{Coords, Number as Nt, Letter as Lt};

use super::{Piece};

#[derive(Debug, Copy, Clone)]
enum Token {
    Invalid,
    Capital(Piece),
    Letter(Lt),
    Number(Nt),
    /// x
    Capture,
    /// +
    Check,
    /// #
    Mate,
    /// 0-0 (O-O)
    Castle,
    /// -0 (-O)
    Long,
    /// =
    Promote,
}

struct TokenStream<'a> {
    chars: Chars<'a>,
    peeked: Option<Token>,
}

impl<'a> TokenStream<'a> {
    fn new(s: &'a str) -> Self {
        TokenStream { chars: s.chars(), peeked: None }
    }
    fn peek(&mut self) -> Option<Token> {
        self.peeked = self.next();
        self.peeked
    }
    fn set_to_peek(&mut self, token: Token) {
        assert!(self.peeked.is_none(), "cannot append on peeked stream");
        self.peeked = Some(token);
    } 
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Token::*;

        let peeked = self.peeked.take();
        if peeked.is_some() {
            return peeked;
        }

        Some(match self.chars.next()? {
            'R' => Capital(Piece::Rook),
            'N' => Capital(Piece::Knight),
            'B' => Capital(Piece::Bishop),
            'Q' => Capital(Piece::Queen),
            'K' => Capital(Piece::King),
            l @ 'a'..='h' => Letter(Lt::from_char(l).unwrap()),
            n @ '1'..='8' => Number(Nt::from_char(n).unwrap()),
            'x' => Capture,
            '+' => Check,
            '#' => Mate,
            '0' | 'O' => {
                match (self.chars.next(), self.chars.next()) {
                    (Some('-'), Some('0' | 'O')) => Castle,
                    _ => Invalid,
                }
            }
            '-' => match self.chars.next() {
                Some('0' | 'O') => Long,
                _ => Invalid,
            }
            '=' => Promote,
            c if c.is_whitespace() => self.next()?,
            _ => Invalid,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub move_type: MoveType,
    pub king_threat: KingThreat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mover {
    Piece(Piece),
    PieceAt(Piece, Coords),
    PieceAtLetter(Piece, Lt),
    PieceAtNumber(Piece, Nt),
    // Coords(Coords),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    ShortCastle,
    LongCastle,
    Regular {
        mover: Mover,
        captures: bool,
        destination: Coords,
        promotes: Option<Piece>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KingThreat {
    None,
    Check,
    CheckMate,
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.move_type {
            MoveType::ShortCastle => write!(f, "O-O")?,
            MoveType::LongCastle => write!(f, "O-O-O")?,
            MoveType::Regular { mover, captures, destination, promotes } => {
                match mover {
                    Mover::Piece(p) => write!(f, "{}", p)?,
                    Mover::PieceAtNumber(p, n) => write!(f, "{}{}", p, n)?,
                    Mover::PieceAtLetter(p, l) => write!(f, "{}{}", p, l)?,
                    Mover::PieceAt(p, cs) => write!(f, "{}{}", p, cs)?,
                }
                if captures { write!(f, "x")?; }
                write!(f, "{}", destination)?;
                if let Some(p) = promotes {
                    write!(f, "={}", p)?;
                }
            }
        }
        match self.king_threat {
            KingThreat::None => Ok(()),
            KingThreat::Check => write!(f, "+"),
            KingThreat::CheckMate => write!(f, "#"),
        }
    }
}

impl MoveType {
    fn parse_regular(piece: Piece, ts: &mut TokenStream) -> Option<Self> {
        use self::Token::*;
        match (ts.next()?, ts.next()?) {
            // Pl -> ln
            (Letter(l), Letter(l2)) => match ts.next()? {
                Number(n) => Some(MoveType::Regular {
                    mover: Mover::PieceAtLetter(piece, l),
                    captures: false,
                    destination: Coords::new(l2, n),
                    promotes: Self::parse_promotion(ts),
                }),
                _ => None,
            }
            // Pn l
            (Number(n), Letter(l)) => match ts.next()? {
                // Pn -> ln 
                Number(n2) => Some(MoveType::Regular {
                    mover: Mover::PieceAtNumber(piece, n),
                    captures: false,
                    destination: Coords::new(l, n2),
                    promotes: Self::parse_promotion(ts),
                }),
                _ => None,
            }
            // Pl.n
            (Letter(l), Number(n)) => match ts.peek() {
                // Pln -> ln
                Some(Letter(l2)) => {
                    ts.peeked = None;
                    match ts.next() {
                        Some(Number(n2)) => Some(MoveType::Regular {
                            mover: Mover::PieceAt(piece, Coords::new(l, n)),
                            captures: false,
                            destination: Coords::new(l2, n2),
                            promotes: Self::parse_promotion(ts),
                        }),
                        _ => None,
                    }
                }
                // Pln x ln
                Some(Capture) => {
                    ts.peeked = None;
                    Some(MoveType::Regular {
                        mover: Mover::PieceAt(piece, Coords::new(l, n)),
                        captures: true,
                        destination: Self::parse_destination(ts)?,
                        promotes: Self::parse_promotion(ts),
                    })
                }
                // P -> ln
                _ => Some(MoveType::Regular {
                    mover: Mover::Piece(piece),
                    captures: false,
                    destination: Coords::new(l, n),
                    promotes: Self::parse_promotion(ts),
                }),
            }
            // at letter capturing -> LN
            (Letter(l), Capture) => Some(MoveType::Regular {
                mover: Mover::PieceAtLetter(piece, l),
                captures: true,
                destination: Self::parse_destination(ts)?,
                promotes: Self::parse_promotion(ts),
            }),
            // at number capturing -> LN
            (Number(n), Capture) => Some(MoveType::Regular {
                mover: Mover::PieceAtNumber(piece, n),
                captures: true,
                destination: Self::parse_destination(ts)?,
                promotes: Self::parse_promotion(ts),
            }),
            // Capture -> LN
            (Capture, Letter(l2)) => match ts.next() {
                Some(Number(n2)) => Some(MoveType::Regular {
                    mover: Mover::Piece(piece),
                    captures: true,
                    destination: Coords::new(l2, n2),
                    promotes: Self::parse_promotion(ts),
                }),
                _ => None
            }
            _ => None,
        }
    }
    fn parse_destination(ts: &mut TokenStream) -> Option<Coords> {
        match (ts.next()?, ts.next()?) {
            (Token::Letter(l), Token::Number(n)) => Some(Coords::new(l, n)),
            _ => None,
        }
    }
    fn parse_promotion(ts: &mut TokenStream) -> Option<Piece> {
        match ts.peek() {
            Some(Token::Promote) => {
                ts.peeked = None;
                match ts.next() {
                    Some(Token::Capital(p)) => Some(p),
                    _ => None,
                }
            },
            _ => None,
        }
    }
    fn from_ts(ts: &mut TokenStream) -> Option<Self> {
        use self::Token::*;
        match ts.next()? {
            Capital(moving_piece) => Self::parse_regular(moving_piece, ts),
            t @ Letter(_) | t @ Number(_) | t @ Capture => {
                ts.set_to_peek(t);
                Self::parse_regular(Piece::Pawn, ts)
            }
            Castle => match ts.peek() {
                Some(Long) => {
                    ts.next();
                    Some(MoveType::LongCastle)
                },
                _ => Some(MoveType::ShortCastle),
            }
            _ => None,
        }
    }
}

impl Move {
    pub fn from_str(s: &str) -> Option<Self> {
        use self::Token::*;
        let mut ts = TokenStream::new(s);

        let move_type = MoveType::from_ts(&mut ts)?;

        Some(match ts.peek() {
            Some(Mate) => Move { move_type, king_threat: KingThreat::CheckMate },
            Some(Check) => {
                let _ = ts.next();

                if matches!(ts.peek(), Some(Check)) {
                    ts.next();
                    Move { move_type, king_threat: KingThreat::CheckMate }
                } else {
                    Move { move_type, king_threat: KingThreat::Check }
                }
            }
            _ => Move { move_type, king_threat: KingThreat::None },
        })
    }
}