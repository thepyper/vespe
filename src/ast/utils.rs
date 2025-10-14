
struct Patches
{
    patches: BTreeMap<(usize, usize), String>,
}

impl Patches
{
    pub fn apply(document: &str, patches: &Patches) -> String {
        // TODO riconstruisce la stringa document patchata, pezzo per pezzo.
        // io penso ad un ciclo che in maniera ordinata dalla prima patch all'ultima
        // incolli progressvamente il pezzo utile di document, poi la patch, poi continui dal successivo
        // pezzo utile di document e cosi via fino ad aver ricostruito la nuova stringa.
        
    }
}