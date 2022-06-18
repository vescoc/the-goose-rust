pub use core::*;

use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SamplePosition(u32);

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Hash, Eq, Debug)]
pub struct SamplePlayer<T>(pub T);

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

impl<Player> State<SamplePlayer<Player>, SamplePosition> for BTreeMap<SamplePlayer<Player>, SamplePosition>
where
    Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
{
    type Error = std::convert::Infallible;

    fn get_player_position(&self, player: &SamplePlayer<Player>) -> Result<Option<SamplePosition>, Self::Error> {
        Ok(self.get(player).copied())
    }

    fn add_player(&mut self, player: SamplePlayer<Player>) -> Result<(), Self::Error> {
        self.insert(player, SamplePosition::from(0));

        Ok(())
    }

    fn remove_player(&mut self, player: &SamplePlayer<Player>) -> Result<(), Self::Error> {
        self.remove(player);

        Ok(())
    }

    fn find_players_by_position(&self, position: &SamplePosition) -> Result<Vec<SamplePlayer<Player>>, Self::Error> {
        Ok(self
            .iter()
            .filter_map(|(k, p)| if p == position { Some(k) } else { None })
            .cloned()
            .collect())
    }

    fn players(&self) -> Result<Vec<SamplePlayer<Player>>, Self::Error> {
        Ok(self.keys().cloned().collect())
    }

    fn update_player_position(
        &mut self,
        player: &SamplePlayer<Player>,
        position: &SamplePosition,
    ) -> Result<(), Self::Error> {
        if let Some(p) = self.get_mut(player) {
            *p = position.clone();
        }

        Ok(())
    }
}

impl<Player> TheGoose<SamplePlayer<Player>, SamplePosition, u32> for BTreeMap<SamplePlayer<Player>, SamplePosition>
where
    Player: Eq + std::hash::Hash + Clone + std::cmp::Ord,
{
    type State = BTreeMap<SamplePlayer<Player>, SamplePosition>;

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
    state: BTreeMap<SamplePlayer<&'a str>, SamplePosition>,
    rolls: I,
}

impl<'a, I> SimpleTheGoose<'a, I> {
    pub fn new(rolls: I) -> Self {
        SimpleTheGoose {
            state: BTreeMap::new(),
            rolls,
        }
    }
}

impl<'a, I: Iterator<Item = u32>> TheGoose<SamplePlayer<&'a str>, SamplePosition, u32>
    for SimpleTheGoose<'a, I>
{
    type State = BTreeMap<SamplePlayer<&'a str>, SamplePosition>;

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
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        assert_eq!(
            the_goose.execute(Command::Add(SamplePlayer("Pippo"))),
            Ok(vec![Event::Players(vec![SamplePlayer("Pippo")])])
        );

        assert_eq!(
            the_goose.execute(Command::Add(SamplePlayer("Pluto"))),
            Ok(vec![Event::Players(vec![SamplePlayer("Pippo"), SamplePlayer("Pluto")])])
        );
    }

    #[test]
    fn test_scenario_1_2() {
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        the_goose.execute(Command::Add(SamplePlayer("Pippo"))).ok();

        assert_eq!(
            the_goose.execute(Command::Add(SamplePlayer("Pippo"))),
            Err(Error::DuplicatePlayer(SamplePlayer("Pippo")))
        );
    }

    #[test]
    fn test_scenario_1_3() {
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        the_goose.execute(Command::Add(SamplePlayer("Pippo"))).ok();

        assert_eq!(
            the_goose.execute(Command::Remove(SamplePlayer("Pippo"))),
            Ok(vec![Event::Players(Vec::new())])
        );
    }

    #[test]
    fn test_scenario_2_1() {
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        the_goose.execute(Command::Add(SamplePlayer("Pippo"))).ok();
        the_goose.execute(Command::Add(SamplePlayer("Pluto"))).ok();

        assert_eq!(
            the_goose.execute(Command::Move(SamplePlayer("Pippo"), 4, 2)),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 4, 2),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(0), SamplePosition(6)),
                Event::Jump(SamplePlayer("Pippo"), SamplePosition(12))
            ])
        );

        assert_eq!(
            the_goose.execute(Command::Move(SamplePlayer("Pluto"), 2, 2)),
            Ok(vec![
                Event::Roll(SamplePlayer("Pluto"), 2, 2),
                Event::Moved(SamplePlayer("Pluto"), SamplePosition(0), SamplePosition(4))
            ])
        );

        assert_eq!(
            the_goose.execute(Command::Move(SamplePlayer("Pippo"), 2, 3)),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 2, 3),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(12), SamplePosition(17))
            ])
        );
    }

    #[test]
    fn test_scenario_3_1() {
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        the_goose.insert(SamplePlayer("Pippo"), SamplePosition(60));

        assert_eq!(
            the_goose.execute(Command::Move(SamplePlayer("Pippo"), 1, 2)),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 1, 2),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(60), SamplePosition(63)),
                Event::Win(SamplePlayer("Pippo"))
            ])
        );
    }

    #[test]
    fn test_scenario_3_2() {
        let mut the_goose = BTreeMap::<_, SamplePosition>::new();

        the_goose.insert(SamplePlayer("Pippo"), SamplePosition(60));

        assert_eq!(
            the_goose.execute(Command::Move(SamplePlayer("Pippo"), 3, 2)),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 3, 2),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(60), SamplePosition(63)),
                Event::Bounced(SamplePlayer("Pippo")),
                Event::Return(SamplePlayer("Pippo"), SamplePosition(62))
            ])
        );
    }

    #[test]
    fn test_scenario_4_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 2].into_iter());

        the_goose.state.insert(SamplePlayer("Pippo"), SamplePosition(3));

        assert_eq!(
            the_goose.execute(Command::RollAndMove(SamplePlayer("Pippo"))),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 1, 2),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(3), SamplePosition(6)),
                Event::Jump(SamplePlayer("Pippo"), SamplePosition(12))
            ])
        );
    }

    #[test]
    fn test_scenario_5_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert(SamplePlayer("Pippo"), SamplePosition(4));

        assert_eq!(
            the_goose.execute(Command::RollAndMove(SamplePlayer("Pippo"))),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 1, 1),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(4), SamplePosition(6)),
                Event::Jump(SamplePlayer("Pippo"), SamplePosition(12))
            ])
        );
    }

    #[test]
    fn test_scenario_6_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert(SamplePlayer("Pippo"), SamplePosition(3));

        assert_eq!(
            the_goose.execute(Command::RollAndMove(SamplePlayer("Pippo"))),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 1, 1),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(3), SamplePosition(5)),
                Event::MovedAgain(SamplePlayer("Pippo"), SamplePosition(5), SamplePosition(7))
            ])
        );
    }

    #[test]
    fn test_scenario_6_2() {
        let mut the_goose = SimpleTheGoose::new(vec![2, 2].into_iter());

        the_goose.state.insert(SamplePlayer("Pippo"), SamplePosition(10));

        assert_eq!(
            the_goose.execute(Command::RollAndMove(SamplePlayer("Pippo"))),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 2, 2),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(10), SamplePosition(14)),
                Event::MovedAgain(SamplePlayer("Pippo"), SamplePosition(14), SamplePosition(18)),
                Event::MovedAgain(SamplePlayer("Pippo"), SamplePosition(18), SamplePosition(22))
            ])
        );
    }

    #[test]
    fn test_scenario_7_1() {
        let mut the_goose = SimpleTheGoose::new(vec![1, 1].into_iter());

        the_goose.state.insert(SamplePlayer("Pippo"), SamplePosition(15));
        the_goose.state.insert(SamplePlayer("Pluto"), SamplePosition(17));

        assert_eq!(
            the_goose.execute(Command::RollAndMove(SamplePlayer("Pippo"))),
            Ok(vec![
                Event::Roll(SamplePlayer("Pippo"), 1, 1),
                Event::Moved(SamplePlayer("Pippo"), SamplePosition(15), SamplePosition(17)),
                Event::Prank(SamplePlayer("Pluto"), SamplePosition(17), SamplePosition(15))
            ])
        );
    }
}
