mod reader;
mod processor;

fn main() {
    reader::read().unwrap();
    let blank = "----.-----.--------.-----.---------------------------..-----....---.----...------.----.-------..-----.---------------------.-----..-------.----.------...----.---....-----..---------------------------.-----.--------.-----.----";
    let solution = "LADD.OPENS.ABBEOLEO.FRATS.SEALAVERYFISHERHALLMARMOSET..EERIE....YES.DABS...TEAPOT.AURA.OPTARLO..ERECT.PAWNATIONALTHEATREGSA.LOVES..NIKEOER.IDES.CANCAN...IVES.BAD....BERNE..PANDEMICTEATROALLASCALAURGE.ICIER.HILTSOAR.LEERY.ODES";
    let empty_board = processor::process_boards(blank);
    let solution_board = processor::process_boards(solution);
}
