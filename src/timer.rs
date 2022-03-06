extern crate time;
extern crate crossbeam_channel;
use std::time::Duration;
use mech_core::*;
use mech_utilities::*;
//use std::sync::mpsc::{self, Sender};
use std::thread::{self};
use crossbeam_channel::Sender;
use std::collections::HashMap;

lazy_static! {
  static ref TIME_TIMER: u64 = hash_str("time/timer");
  static ref PERIOD: u64 = hash_str("period");
  static ref TICKS: u64 = hash_str("ticks");
}

export_machine!(time_timer, time_timer_reg);

extern "C" fn time_timer_reg(registrar: &mut dyn MachineRegistrar, outgoing: Sender<RunLoopMessage>) -> String {
  registrar.register_machine(Box::new(Timer{outgoing, timers: HashMap::new()}));
  "#time/timer = [|period<ms> ticks<u64>|]".to_string()
}

#[derive(Debug)]
pub struct Timer {
  outgoing: Sender<RunLoopMessage>,
  timers: HashMap<usize, (Value,std::thread::JoinHandle<()>)>,
}

impl Machine for Timer {

  fn name(&self) -> String {
    "time/timer".to_string()
  }

  fn id(&self) -> u64 {
    hash_str(&self.name())
  }

  fn on_change(&mut self, table: &Table) -> Result<(), MechError> {
    for i in 1..=table.rows {
      match self.timers.get(&i) {
        Some(timer) => {

        }
        None => {
          let value = table.get(&TableIndex::Index(i), &TableIndex::Alias(*PERIOD))?;
          match value {
            Value::Time(period) => {
              let outgoing = self.outgoing.clone();
              let timer_row = TableIndex::Index(i);
              let timer_handle = thread::spawn(move || {
                let duration = Duration::from_millis((period.unwrap() * 1000.0) as u64);
                let mut counter = 0;
                loop {
                  thread::sleep(duration);
                  counter = counter + 1;
                  outgoing.send(RunLoopMessage::Transaction(vec![
                    Change::Set((*TIME_TIMER,vec![(timer_row, TableIndex::Alias(*TICKS), Value::U64(U64::new(counter)))]))
                  ]));
                }
              });
              self.timers.insert(i,(value,timer_handle));
            }
            _ => return Err(MechError{id: 4782, kind: MechErrorKind::None}),
          }
        }
      }
    }
    Ok(())
  }

}