
struct Patches
{
    patches: BTreeMap<(usize, usize), String>, // (begin_offset, end_offset), replace_string
}

impl Patches
{
    pub fn new() -> Self {
        Patches {
            patches: BTreeMap::new(),
        }
    }
    pub fn add(begin_offset: usize, end_offset: usize, replace: &str) -> Result<(), TODO error > {
        // aggiungi all'insieme delle patches SOLO se non si sovrappone a NESSUNA altra patch presente. se no errore!
    }
    pub fn apply(document: &str, patches: &Patches) -> String {
        // TODO riconstruisce la stringa document patchata, pezzo per pezzo.
        // io penso ad un ciclo che in maniera ordinata dalla prima patch all'ultima
        // incolli progressvamente il pezzo utile di document, poi la patch, poi continui dal successivo
        // pezzo utile di document e cosi via fino ad aver ricostruito la nuova stringa.

    }
}