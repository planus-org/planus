use codespan::FileId;
use codespan_reporting::diagnostic::Label;
use planus_types::ast::Schema;

use crate::{
    ast_convert::ConverterOptions,
    ctx::Ctx,
    util::sorted_map::{sorted_map, SortedMap, SortedSet},
};

#[derive(Default)]
pub struct AstMap {
    asts: SortedMap<FileId, (Schema, Vec<FileId>)>,
    converter_options: ConverterOptions,
}

impl AstMap {
    pub fn new_with_options(converter_options: ConverterOptions) -> Self {
        Self {
            asts: Default::default(),
            converter_options,
        }
    }

    pub fn add_files_recursively(&mut self, ctx: &mut Ctx, file_id: FileId) {
        let mut queue = vec![file_id];

        while let Some(file_id) = queue.pop() {
            match self.asts.entry(file_id) {
                sorted_map::Entry::Occupied(_) => continue,
                sorted_map::Entry::Vacant(entry) => {
                    if let Some(cst) = ctx.parse_file(file_id) {
                        let ast =
                            crate::ast_convert::convert(ctx, file_id, cst, self.converter_options);
                        let dependencies = ast
                            .includes
                            .iter()
                            .filter_map(|literal| {
                                ctx.add_relative_path(
                                    file_id,
                                    &literal.value,
                                    [Label::primary(file_id, literal.span)],
                                )
                            })
                            .collect::<Vec<_>>();
                        queue.extend_from_slice(&dependencies);
                        entry.insert((ast, dependencies));
                    }
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Schema> {
        self.asts.values().map(|(schema, _)| schema)
    }

    pub fn reachability(&self) -> SortedMap<FileId, SortedSet<FileId>> {
        let size = self.asts.len();
        let mut reachability: Vec<bool> = vec![false; size * size];
        macro_rules! get {
            ($i:expr, $j:expr) => {
                &mut reachability[$i * size + $j]
            };
        }

        for (i, (_schema, dependencies)) in self.asts.values().enumerate() {
            *get!(i, i) = true;
            for file_id in dependencies {
                if let Some(j) = self.asts.index_of(file_id) {
                    *get!(i, j) = true;
                }
            }
        }

        for k in 0..size {
            for i in 0..size {
                for j in 0..size {
                    if *get!(i, k) && *get!(k, j) {
                        *get!(i, j) = true;
                    }
                }
            }
        }

        let mut result = SortedMap::new();
        for (i, (key, _)) in self.asts.iter().enumerate() {
            let mut can_reach: SortedSet<FileId> = SortedSet::new();
            for j in 0..size {
                if *get!(i, j) {
                    can_reach.insert(self.asts.0[j].0);
                }
            }
            result.insert(*key, can_reach);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use codespan::Files;

    use super::*;

    #[test]
    fn test_floyd_warshall() {
        let mut files = Files::default();
        let file_id1 = files.add("", "");
        let file_id2 = files.add("", "");
        let file_id3 = files.add("", "");
        for (file_id1, file_id2, file_id3) in [
            (file_id1, file_id2, file_id3),
            (file_id1, file_id3, file_id2),
            (file_id2, file_id1, file_id3),
            (file_id3, file_id1, file_id2),
            (file_id2, file_id3, file_id1),
            (file_id3, file_id2, file_id1),
        ] {
            let mut asts = SortedMap::new();
            asts.insert(
                file_id1,
                (Schema::new(file_id1, String::new()), vec![file_id2]),
            );
            asts.insert(
                file_id2,
                (Schema::new(file_id2, String::new()), vec![file_id3]),
            );
            asts.insert(file_id3, (Schema::new(file_id3, String::new()), vec![]));
            let reachability = AstMap {
                asts,
                converter_options: Default::default(),
            }
            .reachability();
            let mut file_id1_reach = [file_id1, file_id2, file_id3];
            let mut file_id2_reach = [file_id2, file_id3];
            let file_id3_reach = [file_id3];
            file_id1_reach.sort();
            file_id2_reach.sort();

            assert!(reachability
                .get(&file_id1)
                .unwrap()
                .iter()
                .eq(&file_id1_reach));
            assert!(reachability
                .get(&file_id2)
                .unwrap()
                .iter()
                .eq(&file_id2_reach));
            assert!(reachability
                .get(&file_id3)
                .unwrap()
                .iter()
                .eq(&file_id3_reach));
        }
    }
}
