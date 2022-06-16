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
                events.push(Event::Prank(p.clone(), end_position, initial_position));
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

pub mod sample;
