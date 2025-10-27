use anyhow::Result;
use uuid::Uuid;

use super::execute::Collector;
use super::execute::Worker;
use super::variables::Variables;
use super::content::ModelContent;

use crate::ast2::{Anchor, Position, Range, Tag};

// 1. TRAIT BASE
// Definisce l'interfaccia comune per tutti i comportamenti dei tag.
// Nota che tutti i metodi ora ricevono `&self`.
pub trait TagBehavior {
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)>;

    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>>;

    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)>;

    fn collect_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        anchor_end: Position,
    ) -> Result<Option<Collector>>;
}

// 2. MARKER TRAITS E LOGICA SPECIFICA
// Questi trait vuoti servono a "marcare" una struct come statica o dinamica.
// Includono anche i metodi che le implementazioni concrete DEVONO fornire.

trait StaticTag {
    fn collect_static_tag(&self, worker: &Worker, collector: Collector, tag: &Tag) -> Result<Option<Collector>>;
}

trait DynamicTag {
    type State: Default + std::fmt::Debug;
    fn mono(&self, worker: &Worker, collector: Collector, state: Self::State) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)>;
}


// 3. BLANKET IMPLEMENTATIONS
// Qui forniamo un'implementazione di `TagBehavior` per QUALSIASI tipo `T`
// che sia marcato come `StaticTag` o `DynamicTag`.

// Blanket implementation per tutti i tag statici.
impl<T: StaticTag> TagBehavior for T {
    fn execute_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        panic!("StaticTag does not support execute_anchor");
    }

    fn collect_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<Option<Collector>> {
        panic!("StaticTag does not support collect_anchor");
    }

    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let collector = self.collect_tag(worker, collector, tag)?;
        Ok((collector, vec![]))
    }

    fn collect_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        // La logica di default è delegare al metodo specifico del marker trait.
        self.collect_static_tag(worker, collector, tag)
    }
}

// Blanket implementation per tutti i tag dinamici.
impl<T: DynamicTag> TagBehavior for T {
    fn execute_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        let state = T::State::default();
        let (collector, new_state, new_output) = self.mono(worker, collector, state)?;
        if let Some(new_state) = new_state {
            // worker.save_state(new_state, &tag.command.to_string(), &Uuid::new_v4())?;
        }
        let patches = match new_output {
            // Some(output) => worker.patch_tag_to_anchor(tag, &output)?,
            Some(output) => vec![(tag.range, output)], // Sostituito per compilare
            None => vec![],
        };
        Ok((collector, patches))
    }

    fn collect_tag(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _tag: &Tag,
    ) -> Result<Option<Collector>> {
        // I tag dinamici non supportano collect_tag
        Ok(None)
    }

    fn execute_anchor(
        &self,
        worker: &Worker,
        collector: Collector,
        anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<(Option<Collector>, Vec<(Range, String)>)> {
        // let state = worker.load_state::<T::State>(&anchor.command, &anchor.uuid)?;
        // let (collector, new_state, new_output) = self.mono(worker, collector, state)?;
        // if let Some(new_state) = new_state {
        //     worker.save_state(new_state, &anchor.command.to_string(), &anchor.uuid)?;
        // }
        // let patches = match new_output {
        //     Some(output) => worker.patch_into_anchor(anchor, anchor_end, &output)?,
        //     None => vec![],
        // };
        // Ok((collector, patches))
        unimplemented!()
    }

    fn collect_anchor(
        &self,
        _worker: &Worker,
        _collector: Collector,
        _anchor: &Anchor,
        _anchor_end: Position,
    ) -> Result<Option<Collector>> {
        unimplemented!()
    }
}


// 4. IMPLEMENTAZIONI CONCRETE (ora molto più pulite)

#[derive(Debug, Default)]
enum AnswerStatus {
    #[default]
    JustCreated,
    Repeat,
    NeedProcessing,
    NeedInjection,
    Completed,
}

#[derive(Debug, Default)]
struct AnswerState {
    pub status: AnswerStatus,
    pub query: ModelContent,
    pub reply: String,
    pub variables: Variables,
}

struct AnswerTagBehavior;
// Marchiamo la struct e implementiamo solo la logica specifica per `DynamicTag`.
impl DynamicTag for AnswerTagBehavior {
    type State = AnswerState;

    fn mono(
        &self,
        worker: &Worker,
        mut collector: Collector,
        mut state: Self::State,
    ) -> Result<(Option<Collector>, Option<Self::State>, Option<String>)> {
        match state.status {
            AnswerStatus::JustCreated => {
                state.status = AnswerStatus::NeedProcessing;
                // state.query = collector.context.clone();
                state.reply = String::new();
                // state.variables = collector.variables.clone();
                Ok((None, Some(state), Some(String::new())))
            }
            AnswerStatus::NeedProcessing => {
                // let response = worker.call_model(&state.query, &state.variables)?;
                // state.reply = response;
                state.status = AnswerStatus::NeedInjection;
                Ok((None, Some(state), None))
            }
            AnswerStatus::NeedInjection => {
                let output = state.reply.clone();
                state.status = AnswerStatus::Completed;
                Ok((None, Some(state), Some(output)))
            }
            AnswerStatus::Completed | AnswerStatus::Repeat => {
                Ok((Some(collector), None, None))
            }
        }
    }
}


#[derive(Debug)]
struct IncludeTagBehaviour;
// Marchiamo la struct e implementiamo solo la logica specifica per `StaticTag`.
impl StaticTag for IncludeTagBehaviour {
    fn collect_static_tag(
        &self,
        worker: &Worker,
        collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        let included_context_name = tag
            .arguments
            .arguments
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing argument for include tag"))?
            .value
            .clone();
        // worker.collect(collector, &included_context_name)
        unimplemented!()
    }
}


struct SetTagBehaviour;
// Marchiamo la struct e implementiamo solo la logica specifica per `StaticTag`.
impl StaticTag for SetTagBehaviour {
    fn collect_static_tag(
        &self,
        _worker: &Worker,
        mut collector: Collector,
        tag: &Tag,
    ) -> Result<Option<Collector>> {
        // Ok(Some(collector.update(&tag.parameters)))
        unimplemented!()
    }
}

/*
#[enum_dispatch(TagBehavior)]
#[derive(Debug)]
pub enum TagBehaviorDispatch {
    Answer(AnswerTagBehavior),
    Include(IncludeTagBehaviour),
    Static(StaticTagBehaviour),
}
*/
