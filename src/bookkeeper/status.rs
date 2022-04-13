use std::collections::HashMap;

use bigdecimal::BigDecimal;
use nu_table::{draw_table, StyledString, Table, TextStyle, Theme};
use toml::value::Table as TomlTable;

use crate::{
    error::Result,
    parser::{Operation, OperationType},
};

#[allow(unused)]
pub enum StatusInfo {
    Complete,
    Summary,
}

pub struct BookkeeperStatus {
    /// Total amount spent.
    pub take_total: BigDecimal,
    /// Total amount received.
    pub put_total: BigDecimal,
    /// List of all operations.
    pub all_operations: Vec<Operation>,
    /// List of put operations.
    pub put_operations: Vec<Operation>,
    /// List of take operations.
    pub take_operations: Vec<Operation>,
    /// Month of this status, in format MM-YYYY.
    pub month: String,
}

fn table_row_from_operation(operation: &Operation) -> Vec<StyledString> {
    let Operation {
        day,
        kind,
        amount,
        description,
    } = operation;

    let (kind_name, _) = kind.name_and_symbol();

    let line: Vec<StyledString> = [
        format!("{day:2}"),
        kind_name.into(),
        format!("{amount:8.2}"),
        description.into(),
    ]
    .into_iter()
    .map(|x| StyledString::new(x, TextStyle::basic_left()))
    .collect();

    line
}

fn table_header_from_column_names(column_names: &[&str]) -> Vec<StyledString> {
    column_names
        .iter()
        .map(|x| StyledString::new(x, TextStyle::default_header()))
        .collect()
}

impl BookkeeperStatus {
    pub fn display_summaries(selfs: Vec<Self>) {
        let table = {
            let header = ["Month", "Incoming", "Outgoing", "Balance"];
            let header = table_header_from_column_names(&header);

            let (mut all_put, mut all_take, mut all_balance) = (
                BigDecimal::default(),
                BigDecimal::default(),
                BigDecimal::default(),
            );

            let mut rows = selfs
                .into_iter()
                .map(|this| {
                    all_put += &this.put_total;
                    all_take += &this.take_total;
                    let balance = &this.put_total - &this.take_total;
                    all_balance += &balance;

                    [
                        this.month,
                        format!("{:8.2}", this.put_total),
                        format!("{:8.2}", this.take_total),
                        format!("{:7.2}", balance),
                    ]
                    .into_iter()
                    .map(|x| StyledString::new(x, TextStyle::basic_left()))
                    .collect::<Vec<_>>()
                })
                .collect::<Vec<Vec<_>>>();

            rows.push(
                [
                    " total".into(),
                    format!("{:8.2}", all_put),
                    format!("{:8.2}", all_take),
                    format!("{:7.2}", all_balance),
                ]
                .into_iter()
                .map(|x| StyledString::new(x, TextStyle::basic_left()))
                .collect(),
            );

            let theme = Theme::compact();

            Table::new(header, rows, theme)
        };

        Self::display_table(&table);
    }

    fn display_table(table: &Table) {
        let screen_width = get_terminal_width();

        // Do not change any colors, yet.
        let colors = HashMap::new();

        // Draw the table into an string
        let output = draw_table(table, screen_width, &colors, false);
        println!("{}", output);
    }

    fn display_summary_table(&self, month: &str) {
        let balance = &self.put_total - &self.take_total;

        let table = {
            let header = ["Month", "Incoming", "Outgoing", "Balance"];
            let header = table_header_from_column_names(&header);

            let rows = vec![
                month.into(),
                format!("{:8.2}", self.put_total),
                format!("{:8.2}", self.take_total),
                format!("{:7.2}", balance),
            ]
            .into_iter()
            .map(|x| StyledString::new(x, TextStyle::basic_left()))
            .collect();

            let theme = Theme::compact();

            Table::new(header, vec![rows], theme)
        };

        Self::display_table(&table);
    }

    fn display_operations_table(&self) {
        let mut all_operations = self.all_operations.clone();
        all_operations.sort_by(|a, b| a.day.cmp(&b.day).then(a.kind.cmp(&b.kind)));

        let table = {
            let header = ["day", "op", "amount", "description"];
            let header = table_header_from_column_names(&header);

            let rows: Vec<Vec<StyledString>> = all_operations
                .iter()
                .map(table_row_from_operation)
                .collect();

            let theme = Theme::compact();

            Table::new(header, rows, theme)
        };

        Self::display_table(&table);
    }

    pub(super) fn display(&self, status_info: StatusInfo, month: &str) {
        self.display_summary_table(month);
        if let StatusInfo::Complete = status_info {
            self.display_operations_table();
        }
    }

    pub(super) fn from_toml_table(table: &TomlTable, month: &str) -> Result<Self> {
        let (take, put) = (
            table["take"].as_array().unwrap(),
            table["put"].as_array().unwrap(),
        );

        let mut all_operations = vec![];
        let mut put_operations = vec![];
        let mut take_operations = vec![];

        for operation in take.iter().chain(put) {
            let operation = operation.as_str().unwrap();
            let operation = Operation::from_str(operation).unwrap();

            all_operations.push(operation.clone());

            match operation.kind {
                OperationType::Withdraw => take_operations.push(operation),
                OperationType::Deposit => put_operations.push(operation),
            }
        }

        let take_total: BigDecimal = take_operations.iter().map(|x| &x.amount).sum();
        let put_total: BigDecimal = put_operations.iter().map(|x| &x.amount).sum();

        Ok(Self {
            take_total,
            put_total,
            all_operations,
            take_operations,
            put_operations,
            month: month.to_string(),
        })
    }
}

fn get_terminal_width() -> usize {
    termion::terminal_size()
        .map(|(width, _height)| width)
        .expect("Could not get the terminal width")
        .into()
}
