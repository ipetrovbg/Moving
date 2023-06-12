use chrono::Utc;
use escpos_rs::{Printer, PrinterProfile, Error, Instruction, command::Font, Justification, PrintData};

use crate::package::Package;

pub fn print_package(package: Package, count: usize) -> Result<(), Error> {
    let printer_details = PrinterProfile::usb_builder(0x0456, 0x0808).build();
    let printer = match Printer::new(printer_details) {
        Ok(maybe_printer) => match maybe_printer {
            Some(printer) => printer,
            None => panic!("No printer was found :(")
        },
        Err(e) => panic!("Error: {}", e)
    };

    let center_instruction = Instruction::text(
        "%center%",
        Font::FontA,
        Justification::Center,
        Some(vec!["%center%".into()].into_iter().collect())
    );

    let print_header = PrintData::builder()
        .replacement("%center%", "******")
        .build();

    let print_title = PrintData::builder()
        .replacement("%center%", "*** MOVING APP ***")
        .build();

    let print_name = PrintData::builder()
        .replacement("%center%", package.name)
        .build();

    let print_description = PrintData::builder()
        .replacement("%center%", package.description)
        .build();


    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();

    printer.instruction(&center_instruction, Some(&print_header)).unwrap();

    printer.println("").unwrap();

    printer.instruction(&center_instruction, Some(&print_title)).unwrap();

    printer.println("").unwrap();
    printer.println("").unwrap();

    match printer.duo_table(("", ""), vec![
        ("Nil", format!("[ {} ]", package.nil)),
        ("ID", package.id.to_string()),
    ]) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }

    printer.println("").unwrap();
    printer.instruction(&center_instruction, Some(&print_header)).unwrap();
    printer.println("").unwrap();

    printer.instruction(&center_instruction, Some(&print_name)).unwrap();

    printer.println("").unwrap();

    printer.instruction(&center_instruction, Some(&print_description)).unwrap();

    printer.println("").unwrap();

    match printer.duo_table(("", ""), vec![
        ("Created On", package.created_at.format("%d/%b/%Y").to_string()),
        ("Printed On", Utc::now().format("%d/%b/%Y").to_string()),
    ]) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }

    match printer.duo_table(("", ""), vec![
        ("Items in this package", format!("{}", count)),
    ]) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }

    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();
    printer.println("").unwrap();

    Ok(())
}
