extern crate time;
extern crate crossbeam_channel;
use std::time::Duration;
use mech_core::{hash_string, TableIndex, Table, Value, ValueMethods, Transaction, Change, TableId, Register};
use mech_utilities::{Machine, MachineRegistrar, RunLoopMessage};
//use std::sync::mpsc::{self, Sender};
use std::thread::{self};
use crossbeam_channel::Sender;
use std::collections::HashMap;

lazy_static! {
  static ref TIME_TIMER: u64 = hash_string("time/timer");
  static ref PERIOD: u64 = hash_string("period");
  static ref TICKS: u64 = hash_string("ticks");
}

export_machine!(time_timer, time_timer_reg);

extern "C" fn time_timer_reg(registrar: &mut dyn MachineRegistrar, outgoing: Sender<RunLoopMessage>) -> String {
  registrar.register_machine(Box::new(Timer{outgoing, timers: HashMap::new()}));
  "#time/timer = [|period ticks|]".to_string()
}

#[derive(Debug)]
pub struct Timer {
  outgoing: Sender<RunLoopMessage>,
  timers: HashMap<usize, (u64,std::thread::JoinHandle<()>)>,
}

impl Machine for Timer {

  fn name(&self) -> String {
    "time/timer".to_string()
  }

  fn id(&self) -> u64 {
    Register{table_id: TableId::Global(*TIME_TIMER), row: TableIndex::All, column: TableIndex::Alias(*PERIOD)}.hash()
  }

  fn on_change(&mut self, table: &Table) -> Result<(), String> {
    for i in 1..=table.rows {
      match self.timers.get(&i) {
        Some(timer) => {

        }
        None => {
          let (value,_) = table.get(&TableIndex::Index(i), &TableIndex::Alias(*PERIOD)).unwrap();
          match value.as_u64() {
            Some(duration) => {
              let outgoing = self.outgoing.clone();
              let timer_row = TableIndex::Index(i);
              let timer_handle = thread::spawn(move || {
                let duration = Duration::from_millis(duration);
                let mut counter = 0;
                loop {
                  thread::sleep(duration);
                  counter = counter + 1;
                  outgoing.send(RunLoopMessage::Transaction(Transaction{changes: vec![
                    Change::Set{table_id: *TIME_TIMER, values: vec![(timer_row, TableIndex::Alias(*TICKS), Value::from_u64(counter))]}
                  ]}));
                }
              });
              self.timers.insert(i,(duration,timer_handle));
            }
            None => (),
          }
        }
      }
    }
    Ok(())
  }

}