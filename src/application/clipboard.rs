use serde::{Deserialize, Serialize};

use crate::{id::Id, renderer::vector::*, simulator::*};

use super::{action::Action, selection::*};

#[derive(Serialize, Deserialize, Debug)]
pub enum Clipboard {
    Empty,
    Blocks(Vec<Block>, Vec<Connection>),
    Module(Box<Module>),
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::Empty
    }
}

impl Clipboard {
    pub fn serialize(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|err| err.to_string())
    }

    pub fn deserialize(data: &str) -> Result<Self, String> {
        serde_json::from_str(data).map_err(|err| err.to_string())
    }

    pub fn paste_to(
        &self,
        plot_provider: PlotProvider,
        position: Vector2<f64>,
    ) -> Result<Action, String> {
        if let Clipboard::Blocks(blocks, connections) = self {
            let mut data = (blocks.to_owned(), connections.to_owned());
            data.prepare_pasting(position);
            plot_provider.with_mut(|plot| {
                plot.unhighlight();
                plot.set_selection(Selection::Many(
                    data.0
                        .iter()
                        .map(|block| Selectable::Block(block.id()))
                        .collect(),
                ));
            });
            return Ok(Action::PasteBlocks(plot_provider, data.0, data.1));
        }

        panic!("called `paste_to()` on clipboard != Clipboard::Blocks")
    }
}

impl From<&Plot> for Clipboard {
    fn from(plot: &Plot) -> Self {
        match plot.selection() {
            Selection::Single(Selectable::Block(block_id), _) => {
                if let Some(block) = plot.get_block(*block_id) && !block.unique() {
                    let mut block = block.clone();
                    block.prepare_copying(());
                    Self::Blocks(vec![block], Vec::new())
                } else {
                    Self::Empty
                }
            }
            Selection::Many(blocks) => {
                let selection = blocks
                    .iter()
                    .filter_map(|selectable| selectable.block_id())
                    .filter_map(|block_id| plot.get_block(block_id).filter(|block| !block.unique()));
                let block_ids = selection.clone().map(|block| block.id()).collect::<Vec<BlockID>>();
                let blocks = selection.cloned().collect::<Vec<Block>>();
                let mut data = (blocks, Vec::new());
                data.prepare_copying((plot, block_ids));
                Self::Blocks(data.0, data.1)
            }
            _ => Self::Empty
        }
    }
}

trait Copyable<T> {
    fn prepare_copying(&mut self, data: T) -> &mut Self;
}

impl Copyable<(&Plot, Vec<BlockID>)> for (Vec<Block>, Vec<Connection>) {
    fn prepare_copying(&mut self, data: (&Plot, Vec<BlockID>)) -> &mut Self {
        let plot = data.0;
        let block_ids = data.1;
        let blocks = &mut self.0;
        let connections = &mut self.1;

        blocks.iter_mut().for_each(|block| {
            block.outputs_mut().iter_mut().for_each(|c| {
                if let Some(connection) = c.and_then(|id| plot.get_connection(&id)) {
                    let mut connection = connection.clone();
                    if connection.remove_unselected_branches(&block_ids) {
                        *c = None;
                    } else {
                        connections.push(connection);
                    }
                }
            });

            block.inputs_mut().iter_mut().for_each(|c| {
                if let Some(connection) = c.and_then(|id| plot.get_connection(&id)) {
                    if !block_ids.contains(&connection.origin().block_id()) {
                        *c = None
                    }
                }
            });
        });

        self
    }
}

impl Copyable<()> for Block {
    fn prepare_copying(&mut self, _data: ()) -> &mut Self {
        self.outputs_mut().iter_mut().for_each(|c| *c = None);
        self.inputs_mut().iter_mut().for_each(|c| *c = None);
        self
    }
}

trait Pasteable<T> {
    fn prepare_pasting(&mut self, data: T) -> &mut Self;
}

impl Pasteable<Vector2<f64>> for (Vec<Block>, Vec<Connection>) {
    fn prepare_pasting(&mut self, position: Vector2<f64>) -> &mut Self {
        let min = self
            .0
            .iter()
            .map(|block| block.position())
            .min()
            .unwrap_or_default();
        let offset = Vector2::cast(position) - min;

        self.0.iter_mut().for_each(|block| {
            let new_id = Id::new();
            let old_id = block.id();
            block.set_id(new_id);
            block.set_position(block.position() + offset);
            block.set_highlighted(true);

            self.1
                .iter_mut()
                .for_each(|connection| connection.refactor_id(old_id, new_id));
        });

        self.1.iter_mut().for_each(|connection| {
            let old_id = connection.id();
            let new_id = Id::new();
            connection.set_id(new_id);
            connection.for_each_mut_segment(|segment| {
                if let Some(position) = segment.position() {
                    segment.set_position(*position + offset)
                }
            });

            self.0.iter_mut().for_each(|block| {
                block
                    .connections_mut()
                    .filter_map(|c| c.as_mut())
                    .for_each(|c| {
                        if *c == old_id {
                            *c = new_id;
                        }
                    })
            });
        });

        self
    }
}
