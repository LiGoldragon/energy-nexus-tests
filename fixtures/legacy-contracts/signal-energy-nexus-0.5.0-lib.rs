#![allow(non_camel_case_types)]
macro_rules! wire {
    ($item:item) => {
        #[derive(
            rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash,
        )]
        #[cfg_attr(
            feature = "datom",
            derive(datom_codec::Datomizable, datom_codec::Composing)
        )]
        $item
    };
}
wire! { pub enum Provider { Claude, Codex } }
wire! { pub struct Percent(pub i64); }
wire! { pub struct Instant(pub i64); }
wire! { pub struct Minutes(pub i64); }
wire! { pub enum Name { Session, Week, FableWeek } }
wire! { pub struct Window { pub name: Name, pub used: Percent, pub resets: Instant, pub duration: Minutes } }
wire! { pub struct ResetId(pub String); }
wire! { pub enum ResetStatus { Available, Redeeming, Redeemed } }
wire! { pub struct Reset { pub reset_id: ResetId, pub reset_status: ResetStatus, pub expires: Instant } }
wire! { pub struct Reading { pub provider: Provider, pub allowed: bool, pub window_vector: Vec<Window>, pub reset_vector: Vec<Reset>, pub at: Instant } }
wire! { pub enum Level { Plenty, Fair, Low, Out } }
wire! { pub struct Threshold { pub level: Level, pub left: Percent } }
pub use signal_flow::Address;
wire! { pub struct Recipient(pub Address); }
wire! { pub struct Reminder { pub provider: Provider, pub level: Level, pub recipient: Recipient } }
wire! { pub enum Target { Even, SpendBy(Instant) } }
wire! { pub enum Standing { Over, Under, On } }
wire! { pub struct Pace { pub provider: Provider, pub name: Name, pub target: Target, pub expected: Percent, pub used: Percent, pub standing: Standing, pub per_day: Percent } }
wire! { pub struct Caller { pub process_id: i64, pub user_id: i64 } }
wire! { pub enum Query { Read(Provider), Subscribe, Redeem(Provider) } }
wire! { pub struct Call { pub caller: Caller, pub input: Query } }
wire! { pub struct Read_Data { pub reading: Reading, pub pace_vector: Vec<Pace> } }
wire! { pub struct Changed_Data { pub provider: Provider, pub level: Level, pub reading: Reading } }
wire! { pub enum Refused { ProviderDown(Provider), NoReset, RuleRefused(String) } }
wire! { pub enum Response { Read(Read_Data), Subscribed, Changed(Changed_Data), Redeemed(Reset), Refused(Refused) } }
pub const ETHOS: &str = include_str!("../signal.ethos");
impl signal::Contracted for Call {
    const CONTRACT_SOURCE: &'static str = ETHOS;
}
#[cfg(test)]
mod tests {
    use super::*;
    use ethos_zero::{Actualizing, File, Potential, Validating};
    use signal::{ByteViewable, Restorable, Signal, Signalizable};
    #[test]
    fn validated_binary_round_trip() {
        let value = Call {
            caller: Caller {
                process_id: 1,
                user_id: 2,
            },
            input: Query::Read(Provider::Codex),
        };
        let bytes = value.signalize().unwrap();
        let restored = Signal::<Call>::from(bytes.bytes().to_vec())
            .restore()
            .unwrap();
        assert_eq!(restored, value);
    }

    #[test]
    fn ethos_declaration_checks() {
        let file = Potential::<File>::from(ETHOS).actualize().unwrap();
        file.validate().unwrap();
    }

    fn reminder() -> Reminder {
        Reminder {
            provider: Provider::Codex,
            level: Level::Low,
            recipient: Recipient(Address {
                aspect: signal_flow::Aspect::Mind,
                topic: signal_flow::Topic("energyNexus".into()),
                model: signal_flow::Model::Sol,
            }),
        }
    }

    #[test]
    fn a_reminder_to_a_model_addressed_flow_round_trips_in_binary() {
        let value = reminder();
        let bytes = value.signalize().unwrap();
        let restored = Signal::<Reminder>::from(bytes.bytes().to_vec())
            .restore()
            .unwrap();
        assert_eq!(restored, value);
    }

    #[cfg(feature = "datom")]
    #[test]
    fn a_reminder_names_the_model_in_datom_text() {
        use datom_codec::{Actualizing, Budget, Datomizable, Potential};
        use protos::{Compactable, Protosizable, ReaderBudget};
        let value = reminder();
        let text = value.datomize(Vec::new()).protosize().compact();
        assert!(
            text.contains("Sol"),
            "the recipient carries its model: {text}"
        );
        let mut budget = Budget {
            remaining: 256,
            reader: ReaderBudget::try_from(256).unwrap(),
            depth: 0,
            maximum_depth: 32,
        };
        let restored: Reminder = Potential::<Reminder>::from(text.as_str())
            .actualize(&mut budget)
            .unwrap();
        assert_eq!(restored, value);
    }
}
