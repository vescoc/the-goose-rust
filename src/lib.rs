use std::ops;

pub enum AddPosition<Position> {
    Bounced(Position, Position),
    Normal(Position),
}

#[derive(PartialEq, Debug)]
pub enum Error<Player, StateError> {
    Inner(StateError),
    DuplicatePlayer(Player),
    PlayerNotFound(Player),
}

pub enum Command<Player, Roll> {
    Add(Player),
    Remove(Player),
    Move(Player, Roll, Roll),
    RollAndMove(Player),
}

#[derive(PartialEq, Debug)]
pub enum Event<Player, Position, Roll> {
    Players(Vec<Player>),
    Moved(Player, Position, Position),
    MovedAgain(Player, Position, Position),
    Roll(Player, Roll, Roll),
    Bounced(Player),
    Return(Player, Position),
    Win(Player),
    Prank(Player, Position, Position),
    Jump(Player, Position),
}

impl<Player, E> From<E> for Error<Player, E> {
    fn from(e: E) -> Self {
        Error::Inner(e)
    }
}

pub enum PositionType {
    TheBridge,
    TheGoose,
    Normal,
    End,
}

pub trait Position<Roll>: Sized + std::convert::From<u32> {
    fn add(self, r: Roll) -> AddPosition<Self>;
    fn get_type(&self) -> PositionType;
}

pub trait State<Player, Position> {
    type Error;

    fn get_player_position(&self, player: &Player) -> Result<Option<Position>, Self::Error>;
    fn add_player(&mut self, player: Player) -> Result<(), Self::Error>;
    fn remove_player(&mut self, player: &Player) -> Result<(), Self::Error>;
    fn find_players_by_position(&self, position: &Position) -> Result<Vec<Player>, Self::Error>;
    fn players(&self) -> Result<Vec<Player>, Self::Error>;
    fn update_player_position(
        &mut self,
        player: &Player,
        position: &Position,
    ) -> Result<(), Self::Error>;
}

pub trait TheGoose<Player, P, R> {
    type State: State<Player, P>;

    #[allow(clippy::type_complexity)]
    fn execute(
        &mut self,
        command: Command<Player, R>,
    ) -> Result<
        Vec<Event<Player, P, R>>,
        Error<Player, <<Self as TheGoose<Player, P, R>>::State as State<Player, P>>::Error>,
    >
    where
        Player: Copy,
        R: ops::Add<Output = R> + Copy,
        P: Position<R> + Copy,
    {
        use Command::*;
        use Event::*;

        match command {
            Add(player) => self
                .add_player(player)
                .and_then(|_| Ok(vec![Players(self.state().players()?)])),
            Remove(player) => self
                .remove_player(&player)
                .and_then(|_| Ok(vec![Players(self.state().players()?)])),
            Move(player, dice1, dice2) => self.move_player(&player, dice1, dice2),
            RollAndMove(player) => self.roll_and_move_player(&player),
        }
    }

    #[allow(clippy::type_complexity)]
    fn add_player(
        &mut self,
        player: Player,
    ) -> Result<
        (),
        Error<Player, <<Self as TheGoose<Player, P, R>>::State as State<Player, P>>::Error>,
    > {
        if self.state().get_player_position(&player)?.is_none() {
            Ok(self.state_mut().add_player(player)?)
        } else {
            Err(Error::DuplicatePlayer(player))
        }
    }

    #[allow(clippy::type_complexity)]
    fn remove_player(
        &mut self,
        player: &Player,
    ) -> Result<
        (),
        Error<Player, <<Self as TheGoose<Player, P, R>>::State as State<Player, P>>::Error>,
    > {
        Ok(self.state_mut().remove_player(player)?)
    }

    #[allow(clippy::type_complexity)]
    fn move_player(
        &mut self,
        player: &Player,
        dice1: R,
        dice2: R,
    ) -> Result<
        Vec<Event<Player, P, R>>,
        Error<Player, <<Self as TheGoose<Player, P, R>>::State as State<Player, P>>::Error>,
    >
    where
        R: ops::Add<Output = R> + Copy,
        P: Position<R> + Copy,
        Player: Clone,
    {
        let initial_position = self
            .state()
            .get_player_position(player)?
            .ok_or_else(|| Error::PlayerNotFound(player.clone()))?;

        let mut start_position = initial_position;

        let mut again = false;
        let mut events = vec![Event::Roll(player.clone(), dice1, dice2)];
        loop {
            let end_position = match start_position.add(dice1 + dice2) {
                AddPosition::Bounced(bounced_position, end_position) => {
                    events.push(if again {
                        Event::MovedAgain(player.clone(), start_position, end_position)
                    } else {
                        Event::Moved(player.clone(), start_position, end_position)
                    });
                    events.push(Event::Bounced(player.clone()));
                    events.push(Event::Return(player.clone(), bounced_position));
                    bounced_position
                }
                AddPosition::Normal(end_position) => {
                    events.push(if again {
                        Event::MovedAgain(player.clone(), start_position, end_position)
                    } else {
                        Event::Moved(player.clone(), start_position, end_position)
                    });
                    end_position
                }
            };

            let players = self.state().find_players_by_position(&end_position)?;

            start_position = end_position;

            self.state_mut()
                .update_player_position(player, &start_position)?;

            for p in players {
                events.push(Event::Return(p.clone(), initial_position));
                self.state_mut()
                    .update_player_position(&p, &initial_position)?;
            }

            match start_position.get_type() {
                PositionType::TheBridge => {
                    start_position = 12.into();
                    self.state_mut()
                        .update_player_position(player, &start_position)?;
                    events.push(Event::Jump(player.clone(), start_position));
                    break;
                }
                PositionType::TheGoose => {}
                PositionType::Normal => break,
                PositionType::End => {
                    events.push(Event::Win(player.clone()));
                    break;
                }
            }

            again = true;
        }

        Ok(events)
    }

    #[allow(clippy::type_complexity)]
    fn roll_and_move_player(
        &mut self,
        player: &Player,
    ) -> Result<
        Vec<Event<Player, P, R>>,
        Error<Player, <<Self as TheGoose<Player, P, R>>::State as State<Player, P>>::Error>,
    >
    where
        R: ops::Add<Output = R> + Copy,
        P: Position<R> + Copy,
        Player: Copy,
    {
        let (dice1, dice2) = (self.roll_dice(), self.roll_dice());

        self.move_player(player, dice1, dice2)
    }

    fn state(&self) -> &Self::State;

    fn state_mut(&mut self) -> &mut Self::State;

    fn roll_dice(&mut self) -> R;
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    impl Position<u32> for u32 {
        fn add(self, r: u32) -> AddPosition<Self> {
            if self + r > 63 {
                AddPosition::Bounced(63 * 2 - self - r + 1, 63)
            } else {
                AddPosition::Normal(self + r)
            }
        }

        fn get_type(&self) -> PositionType {
            match self {
                6 => PositionType::TheBridge,
                5 | 9 | 14 | 18 | 23 | 27 => PositionType::TheGoose,
                63 => PositionType::End,
                _ => PositionType::Normal,
            }
        }
    }

    impl<Player, Position> State<Player, Position> for BTreeMap<Player, Position>
    where
        Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
        Position: Copy + std::convert::From<u32> + std::cmp::PartialEq,
    {
        type Error = std::convert::Infallible;

        fn get_player_position(&self, player: &Player) -> Result<Option<Position>, Self::Error> {
            Ok(self.get(player).copied())
        }

        fn add_player(&mut self, player: Player) -> Result<(), Self::Error> {
            self.insert(player, Position::from(0));

            Ok(())
        }

        fn remove_player(&mut self, player: &Player) -> Result<(), Self::Error> {
            self.remove(player);

            Ok(())
        }

        fn find_players_by_position(
            &self,
            position: &Position,
        ) -> Result<Vec<Player>, Self::Error> {
            Ok(self
                .iter()
                .filter_map(|(k, p)| if p == position { Some(k) } else { None })
                .cloned()
                .collect())
        }

        fn players(&self) -> Result<Vec<Player>, Self::Error> {
            Ok(self.keys().cloned().collect())
        }

        fn update_player_position(
            &mut self,
            player: &Player,
            position: &Position,
        ) -> Result<(), Self::Error> {
            if let Some(p) = self.get_mut(player) {
                *p = position.clone();
            }

            Ok(())
        }
    }

    impl<Player, Position> TheGoose<Player, Position, u32> for BTreeMap<Player, Position>
    where
        Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
        Position: Copy + std::convert::From<u32> + std::cmp::PartialEq,
    {
        type State = BTreeMap<Player, Position>;

        fn state(&self) -> &Self::State {
            self
        }

        fn state_mut(&mut self) -> &mut Self::State {
            self
        }

        fn roll_dice(&mut self) -> u32 {
            0
        }
    }

    struct SimpleTheGoose<'a, Position, I> {
        state: BTreeMap<&'a str, Position>,
        rolls: I,
    }

    impl<'a, Position, I> SimpleTheGoose<'a, Position, I> {
        fn new(rolls: I) -> Self {
            SimpleTheGoose {
                state: BTreeMap::new(),
                rolls,
            }
        }
    }

    impl<'a, Position, I: Iterator<Item = u32>> TheGoose<&'a str, Position, u32>
        for SimpleTheGoose<'a, Position, I>
    where
        Position: Copy + std::convert::From<u32> + std::cmp::PartialEq,
    {
        type State = BTreeMap<&'a str, Position>;

        fn state(&self) -> &Self::State {
            &self.state
        }

        fn state_mut(&mut self) -> &mut Self::State {
            &mut self.state
        }

        fn roll_dice(&mut self) -> u32 {
            self.rolls.next().unwrap()
        }
    }

    #[test]
    fn test_scenario_1_1() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        assert_eq!(
            the_goose.execute(Command::Add("Pippo")),
            Ok(vec![Event::Players(vec!["Pippo"])])
        );

        assert_eq!(
            the_goose.execute(Command::Add("Pluto")),
            Ok(vec![Event::Players(vec!["Pippo", "Pluto"])])
        );
    }

    #[test]
    fn test_scenario_1_2() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        the_goose.execute(Command::Add("Pippo")).ok();

        assert_eq!(
            the_goose.execute(Command::Add("Pippo")),
            Err(Error::DuplicatePlayer("Pippo"))
        );
    }

    #[test]
    fn test_scenario_1_3() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        the_goose.execute(Command::Add("Pippo")).ok();

        assert_eq!(
            the_goose.execute(Command::Remove("Pippo")),
            Ok(vec![Event::Players(Vec::new())])
        );
    }

    #[test]
    fn test_scenario_2_1() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        the_goose.execute(Command::Add("Pippo")).ok();
        the_goose.execute(Command::Add("Pluto")).ok();

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 4, 2)),
            Ok(vec![
                Event::Roll("Pippo", 4, 2),
                Event::Moved("Pippo", 0, 6),
                Event::Jump("Pippo", 12)
            ])
        );

        assert_eq!(
            the_goose.execute(Command::Move("Pluto", 2, 2)),
            Ok(vec![
                Event::Roll("Pluto", 2, 2),
                Event::Moved("Pluto", 0, 4)
            ])
        );

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 2, 3)),
            Ok(vec![
                Event::Roll("Pippo", 2, 3),
                Event::Moved("Pippo", 12, 17)
            ])
        );
    }

    #[test]
    fn test_scenario_3_1() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        the_goose.insert("Pippo", 60);

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 1, 2)),
            Ok(vec![
                Event::Roll("Pippo", 1, 2),
                Event::Moved("Pippo", 60, 63),
                Event::Win("Pippo")
            ])
        );
    }

    #[test]
    fn test_scenario_3_2() {
        let mut the_goose = BTreeMap::<_, u32>::new();

        the_goose.insert("Pippo", 60);

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 3, 2)),
            Ok(vec![
                Event::Roll("Pippo", 3, 2),
                Event::Moved("Pippo", 60, 63),
                Event::Bounced("Pippo"),
                Event::Return("Pippo", 62)
            ])
        );
    }

    #[test]
    fn test_scenario_4_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 2].into_iter());

        the_goose.state.insert("Pippo", 3);

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 2),
                Event::Moved("Pippo", 3, 6),
                Event::Jump("Pippo", 12)
            ])
        );
    }

    #[test]
    fn test_scenario_5_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", 4);

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", 4, 6),
                Event::Jump("Pippo", 12)
            ])
        );
    }

    #[test]
    fn test_scenario_6_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", 3);

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", 3, 5),
                Event::MovedAgain("Pippo", 5, 7)
            ])
        );
    }

    #[test]
    fn test_scenario_6_2() {
        let mut the_goose = SimpleTheGoose::new(vec![2, 2].into_iter());

        the_goose.state.insert("Pippo", 10);

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 2, 2),
                Event::Moved("Pippo", 10, 14),
                Event::MovedAgain("Pippo", 14, 18),
                Event::MovedAgain("Pippo", 18, 22)
            ])
        );
    }

    #[test]
    fn test_scenario_7_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", 15);
        the_goose.state.insert("Pluto", 17);

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", 15, 17),
                Event::Return("Pluto", 15)
            ])
        );
    }
}
