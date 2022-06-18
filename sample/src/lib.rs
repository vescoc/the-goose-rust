pub use the_goose::*;

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Debug)]
pub struct SampleEvents<Player, Position, Roll, II: IntoIterator<Item = Player>>(
    Vec<Event<Player, Position, Roll, II>>,
);

impl<Player, Position, Roll, II: IntoIterator<Item = Player>> Deref
    for SampleEvents<Player, Position, Roll, II>
{
    type Target = Vec<Event<Player, Position, Roll, II>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SamplePosition(u32);

#[derive(Default)]
pub struct SampleTheGoose<Player, Position>(BTreeMap<Player, Position>);

impl<Player, Position> SampleTheGoose<Player, Position> {
    pub fn new() -> Self {
        SampleTheGoose(BTreeMap::new())
    }
}

impl<Player, Position> Deref for SampleTheGoose<Player, Position> {
    type Target = BTreeMap<Player, Position>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Player, Position> DerefMut for SampleTheGoose<Player, Position> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Player, Position, Roll, II: IntoIterator<Item = Player>> Default
    for SampleEvents<Player, Position, Roll, II>
{
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<Player, Position, Roll, II: IntoIterator<Item = Player>> Events<Player, Position, Roll, II>
    for SampleEvents<Player, Position, Roll, II>
{
    type Error = std::convert::Infallible;

    fn notify(&mut self, event: Event<Player, Position, Roll, II>) -> Result<(), Self::Error> {
        self.0.push(event);
        Ok(())
    }
}

impl<Player, Position, Roll, II: IntoIterator<Item = Player>>
    From<Vec<Event<Player, Position, Roll, II>>> for SampleEvents<Player, Position, Roll, II>
{
    fn from(v: Vec<Event<Player, Position, Roll, II>>) -> Self {
        Self(v)
    }
}

impl Position<u32> for SamplePosition {
    fn add(self, r: u32) -> AddPosition<Self> {
        if self.0 + r > 63 {
            AddPosition::Bounced(SamplePosition(63 * 2 - self.0 - r + 1), SamplePosition(63))
        } else {
            AddPosition::Normal(SamplePosition(self.0 + r))
        }
    }

    fn get_type(&self) -> PositionType {
        match self.0 {
            6 => PositionType::TheBridge,
            5 | 9 | 14 | 18 | 23 | 27 => PositionType::TheGoose,
            63 => PositionType::End,
            _ => PositionType::Normal,
        }
    }
}

impl std::convert::From<u32> for SamplePosition {
    fn from(value: u32) -> Self {
        SamplePosition(value)
    }
}

impl<Player> State<Player, SamplePosition, Vec<Player>> for SampleTheGoose<Player, SamplePosition>
where
    Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
{
    type Error = std::convert::Infallible;

    fn get_player_position(&self, player: &Player) -> Result<Option<SamplePosition>, Self::Error> {
        Ok(self.get(player).copied())
    }

    fn add_player(&mut self, player: Player) -> Result<(), Self::Error> {
        self.insert(player, SamplePosition::from(0));

        Ok(())
    }

    fn remove_player(&mut self, player: &Player) -> Result<(), Self::Error> {
        self.remove(player);

        Ok(())
    }

    fn find_players_by_position(
        &self,
        position: &SamplePosition,
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
        position: &SamplePosition,
    ) -> Result<(), Self::Error> {
        if let Some(p) = self.get_mut(player) {
            *p = *position;
        }

        Ok(())
    }
}

impl<Player> TheGoose<Player, SamplePosition, u32, Vec<Player>>
    for SampleTheGoose<Player, SamplePosition>
where
    Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
{
    type State = SampleTheGoose<Player, SamplePosition>;
    type Events = SampleEvents<Player, SamplePosition, u32, Vec<Player>>;

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

pub struct SimpleTheGoose<'a, I> {
    state: SampleTheGoose<&'a str, SamplePosition>,
    rolls: I,
}

impl<'a, I> SimpleTheGoose<'a, I> {
    pub fn new(rolls: I) -> Self {
        SimpleTheGoose {
            state: SampleTheGoose(BTreeMap::new()),
            rolls,
        }
    }
}

impl<'a, I: Iterator<Item = u32>> TheGoose<&'a str, SamplePosition, u32, Vec<&'a str>>
    for SimpleTheGoose<'a, I>
{
    type State = SampleTheGoose<&'a str, SamplePosition>;
    type Events = SampleEvents<&'a str, SamplePosition, u32, Vec<&'a str>>;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scenario_1_1() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        assert_eq!(
            the_goose.execute(Command::Add("Pippo")),
            Ok(vec![Event::Players(vec!["Pippo"])].into())
        );

        assert_eq!(
            the_goose.execute(Command::Add("Pluto")),
            Ok(vec![Event::Players(vec!["Pippo", "Pluto"])].into())
        );
    }

    #[test]
    fn test_scenario_1_2() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        the_goose.execute(Command::Add("Pippo")).ok();

        assert_eq!(
            the_goose.execute(Command::Add("Pippo")),
            Err(Error::DuplicatePlayer("Pippo"))
        );
    }

    #[test]
    fn test_scenario_1_3() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        the_goose.execute(Command::Add("Pippo")).ok();

        assert_eq!(
            the_goose.execute(Command::Remove("Pippo")),
            Ok(vec![Event::Players(Vec::new())].into())
        );
    }

    #[test]
    fn test_scenario_2_1() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        the_goose.execute(Command::Add("Pippo")).ok();
        the_goose.execute(Command::Add("Pluto")).ok();

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 4, 2)),
            Ok(vec![
                Event::Roll("Pippo", 4, 2),
                Event::Moved("Pippo", SamplePosition(0), SamplePosition(6)),
                Event::Jump("Pippo", SamplePosition(12))
            ]
            .into())
        );

        assert_eq!(
            the_goose.execute(Command::Move("Pluto", 2, 2)),
            Ok(vec![
                Event::Roll("Pluto", 2, 2),
                Event::Moved("Pluto", SamplePosition(0), SamplePosition(4))
            ]
            .into())
        );

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 2, 3)),
            Ok(vec![
                Event::Roll("Pippo", 2, 3),
                Event::Moved("Pippo", SamplePosition(12), SamplePosition(17))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_3_1() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        the_goose.insert("Pippo", SamplePosition(60));

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 1, 2)),
            Ok(vec![
                Event::Roll("Pippo", 1, 2),
                Event::Moved("Pippo", SamplePosition(60), SamplePosition(63)),
                Event::Win("Pippo")
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_3_2() {
        let mut the_goose = SampleTheGoose::<_, SamplePosition>::new();

        the_goose.insert("Pippo", SamplePosition(60));

        assert_eq!(
            the_goose.execute(Command::Move("Pippo", 3, 2)),
            Ok(vec![
                Event::Roll("Pippo", 3, 2),
                Event::Moved("Pippo", SamplePosition(60), SamplePosition(63)),
                Event::Bounced("Pippo"),
                Event::Return("Pippo", SamplePosition(62))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_4_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 2].into_iter());

        the_goose.state.insert("Pippo", SamplePosition(3));

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 2),
                Event::Moved("Pippo", SamplePosition(3), SamplePosition(6)),
                Event::Jump("Pippo", SamplePosition(12))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_5_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", SamplePosition(4));

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", SamplePosition(4), SamplePosition(6)),
                Event::Jump("Pippo", SamplePosition(12))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_6_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", SamplePosition(3));

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", SamplePosition(3), SamplePosition(5)),
                Event::MovedAgain("Pippo", SamplePosition(5), SamplePosition(7))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_6_2() {
        let mut the_goose = SimpleTheGoose::new(vec![2, 2].into_iter());

        the_goose.state.insert("Pippo", SamplePosition(10));

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 2, 2),
                Event::Moved("Pippo", SamplePosition(10), SamplePosition(14)),
                Event::MovedAgain("Pippo", SamplePosition(14), SamplePosition(18)),
                Event::MovedAgain("Pippo", SamplePosition(18), SamplePosition(22))
            ]
            .into())
        );
    }

    #[test]
    fn test_scenario_7_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert("Pippo", SamplePosition(15));
        the_goose.state.insert("Pluto", SamplePosition(17));

        assert_eq!(
            the_goose.execute(Command::RollAndMove("Pippo")),
            Ok(vec![
                Event::Roll("Pippo", 1, 1),
                Event::Moved("Pippo", SamplePosition(15), SamplePosition(17)),
                Event::Prank("Pluto", SamplePosition(17), SamplePosition(15))
            ]
            .into())
        );
    }
}
