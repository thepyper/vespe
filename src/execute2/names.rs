use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// A curated list of common, multicultural first names (examples, not exhaustive)
const FIRST_NAMES: &[&str] = &[
    "Aaliyah",
    "Aaron",
    "Adam",
    "Aisha",
    "Alex",
    "Alexander",
    "Alice",
    "Aminah",
    "Andrea",
    "Anna",
    "Antonio",
    "Anya",
    "Benjamin",
    "Bianca",
    "Carlos",
    "Catherine",
    "Chloe",
    "Chris",
    "Christian",
    "Daniel",
    "David",
    "Diana",
    "Diego",
    "Elena",
    "Elias",
    "Emily",
    "Enzo",
    "Eric",
    "Eva",
    "Fatima",
    "Felix",
    "Gabriel",
    "Gabriella",
    "George",
    "Hannah",
    "Hector",
    "Isabella",
    "Ivan",
    "Jaden",
    "James",
    "Jasmine",
    "Javier",
    "Jessica",
    "João",
    "John",
    "Joseph",
    "Julia",
    "Kai",
    "Karim",
    "Kimberly",
    "Layla",
    "Leonardo",
    "Liam",
    "Lucas",
    "Maria",
    "Mark",
    "Maya",
    "Mehmet",
    "Mia",
    "Michael",
    "Miguel",
    "Mohammed",
    "Naomi",
    "Natalia",
    "Nathan",
    "Noah",
    "Olivia",
    "Omar",
    "Patricia",
    "Pedro",
    "Penelope",
    "Priya",
    "Rachel",
    "Rafael",
    "Riley",
    "Robert",
    "Rudolf",
    "Ryan",
    "Samantha",
    "Samuel",
    "Sara",
    "Sofia",
    "Sophia",
    "Sora",
    "Stefano",
    "Talia",
    "Thomas",
    "Victor",
    "Victoria",
    "William",
    "Zoe",
    "Zara",
];

// A curated list of common, multicultural last names (examples, not exhaustive)
const LAST_NAMES: &[&str] = &[
    "Almeida",
    "Andersen",
    "Antonova",
    "Chen",
    "Conti",
    "Cruz",
    "Diaz",
    "Dupont",
    "Garcia",
    "Gonzalez",
    "Hernandez",
    "Ivanov",
    "Jackson",
    "Kim",
    "Lee",
    "Lopez",
    "Martinez",
    "Miller",
    "Nguyen",
    "Patel",
    "Pereira",
    "Rossi",
    "Schmidt",
    "Silva",
    "Smith",
    "Suzuki",
    "Wang",
    "Weber",
    "Williams",
    "Wong",
    "Yamamoto",
    "Zhu",
    "Ahmed",
    "Ali",
    "Costa",
    "Da Silva",
    "De Jong",
    "Dubois",
    "Fischer",
    "Gao",
    "Gupta",
    "Hoffmann",
    "Jensen",
    "Johansen",
    "Kimura",
    "Kowalski",
    "Kumar",
    "Leblanc",
    "Li",
    "Martinez",
    "Moreno",
    "Müller",
    "Novak",
    "Oliveira",
    "Peters",
    "Popov",
    "Rahman",
    "Reyes",
    "Santos",
    "Schneider",
    "Singh",
    "Sokolov",
    "Stein",
    "Taylor",
    "Tremblay",
    "Vasquez",
    "Wagner",
    "Wilson",
    "Yoshida",
    "Zhukov",
    "Bauer",
    "Becker",
    "Chang",
    "Chavez",
    "Choi",
    "Chung",
    "Cohen",
    "Dinh",
    "Franco",
    "Fuentes",
    "Gallagher",
    "Gomez",
    "Graham",
    "Hansen",
    "Hart",
    "Hawkins",
    "Henderson",
    "Huang",
    "Huber",
    "Jenkins",
    "Keller",
    "Khan",
    "Kim",
    "Klein",
    "Koch",
    "Kruger",
    "Lambert",
    "Lewis",
    "Liu",
    "Long",
    "Marsh",
    "Meyer",
    "Morales",
    "Nelson",
    "Owens",
    "Park",
    "Pham",
    "Price",
    "Ramirez",
    "Reid",
    "Riley",
    "Rivera",
    "Roberts",
    "Robinson",
    "Rodriguez",
    "Romano",
    "Ruiz",
    "Russell",
    "Scott",
    "Sharp",
    "Shaw",
    "Simpson",
    "Spencer",
    "Stone",
    "Sullivan",
    "Thompson",
    "Tran",
    "Wagner",
    "Walker",
    "Ward",
    "Washington",
    "White",
    "Wright",
    "Young",
    "Zimmerman",
];

/// Generates a name (first name + last name) based on the input content using hashing.
/// Optionally adds a Roman numeral to reduce collision probability.
///
/// The selection of names and the Roman numeral is deterministic based on the input string's hash.
pub fn generate_name(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash_value = hasher.finish();

    let first_name_index = (hash_value % FIRST_NAMES.len() as u64) as usize;
    let last_name_index =
        ((hash_value / FIRST_NAMES.len() as u64) % LAST_NAMES.len() as u64) as usize;
    let roman_numeral_index =
        ((hash_value / (FIRST_NAMES.len() * LAST_NAMES.len()) as u64) % 10) as usize; // 0-9

    let first_name = FIRST_NAMES[first_name_index];
    let last_name = LAST_NAMES[last_name_index];

    let roman_numerals = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
    let roman_numeral = roman_numerals[roman_numeral_index];

    let name = format!("{} {} {}", first_name, last_name, roman_numeral);

    tracing::debug!("hash {} name {}", content, name);

    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_name_consistency() {
        let content1 = "test_string_1";
        let content2 = "test_string_1";
        let content3 = "test_string_2";

        let name1 = generate_name(content1);
        let name2 = generate_name(content2);
        let name3 = generate_name(content3);

        // Ensure that the same content always produces the same name
        assert_eq!(name1, name2);
        // Ensure different content produces different names (highly probable, but not guaranteed due to hash collisions)
        assert_ne!(name1, name3);
    }

    #[test]
    fn test_generate_name_format() {
        let name = generate_name("any_content");
        let parts: Vec<&str> = name.split(' ').collect();

        assert_eq!(
            parts.len(),
            3,
            "Name should have three parts: First_Name Last_Name Roman_Numeral"
        );
        assert!(!parts[0].is_empty(), "First name should not be empty");
        assert!(!parts[1].is_empty(), "Last name should not be empty");
        assert!(!parts[2].is_empty(), "Roman numeral should not be empty");

        let roman_numerals = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
        assert!(
            roman_numerals.contains(&parts[2]),
            "Third part should be a valid Roman numeral"
        );
    }

    #[test]
    fn test_name_and_surname_diversity() {
        // This test doesn't guarantee diversity across all possible hashes,
        // but it checks that different hash values can result in different names.
        let mut generated_names = std::collections::HashSet::new();
        for i in 0..100 {
            // Generate 100 names to check for reasonable diversity
            let name = generate_name(&format!("content_{}", i));
            generated_names.insert(name);
        }
        // With 100 distinct inputs, we expect more than a handful of distinct names
        // based on the current name list sizes (90 first names * 100 last names * 10 numerals = 90000 combinations)
        assert!(
            generated_names.len() > 50,
            "Expected a good number of unique names from diverse inputs"
        );
    }
}
