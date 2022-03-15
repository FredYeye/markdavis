#[derive(PartialEq)]
pub enum Version
{
    Us, Jp,
}

impl Version
{
    fn offset(&self) -> usize
    {
        match self
        {
            Version::Us => 0,
            Version::Jp => 1,
        }
    }
}

const DAYS: usize = 7;

fn main()
{
    let mut rom = Rom::default();

    rom.set_version(Version::Us);
    rom.print_spots();
    rom.print_bait_ratings(false);
    rom.calculate_averages();

    rom.set_version(Version::Jp);
    rom.calculate_averages();
}

pub struct Rom
{
    data: [Vec<u8>; 2],
    version: Version,
}

impl Default for Rom
{
    fn default() -> Self
    {
        Self
        {
            data:
            [
                std::fs::read("Mark Davis' The Fishing Master (USA).sfc").expect("Mark Davis' The Fishing Master (USA).sfc not found!"),
                std::fs::read("Oomono Black Bass Fishing - Jinzouko Hen (Japan).sfc").expect("Oomono Black Bass Fishing - Jinzouko Hen (Japan).sfc not found!"),
            ],

            version: Version::Us,
        }
    }
}

impl Rom
{
    fn get_u8(&self, offset: usize) -> u8
    {
        self.data[self.version.offset()][offset]
    }

    fn make_u16(&self, offset: usize) -> u16 //special case with offset
    {
        u16::from_le_bytes
        (
            [
                self.data[self.version.offset()][ offset | 0x10000     ],
                self.data[self.version.offset()][(offset | 0x10000) + 1],
            ]
        )
    }

    pub fn set_version(&mut self, version: Version)
    {
        self.version = version;
    }

    pub fn print_spots(&self)
    {
        if self.version == Version::Jp
        {
            todo!("jp not implemented for spots")
        }

        let offset = 0xD90B + 0 * 2;

        let stage_offset = self.make_u16(offset) as usize;

        let mut print_string = String::new();

        for area in 0 .. 31
        {
            print_string.push_str(&format!("Area {}\n", area + 1));

            let sub_area = self.make_u16(stage_offset + area * 2) as usize;
            let mut spot_offsets = Vec::new();

            for x in 0 .. 16
            {
                let ofs = self.make_u16(sub_area + x * 2) as usize;
                if ofs > 0xD800
                {
                    spot_offsets.push(ofs);
                }
                else
                {
                    break;
                }
            };

            let mut current_spot = 0;

            while current_spot < spot_offsets.len()
            {
                let mut spot_start = spot_offsets[current_spot];
                let spot_end = if current_spot != spot_offsets.len() - 1
                {
                    spot_offsets[current_spot + 1]
                }
                else
                {
                    if area != 30
                    {
                        self.make_u16(stage_offset + area * 2 + 2) as usize
                    }
                    else // special case for area 30
                    {
                        spot_start + 3 * 2
                    }
                };

                print_string.push_str(&format!("Sub area {} | ", current_spot + 1));

                loop
                {
                    print_string.push_str(&format!("{}", self.make_u16(spot_start) >> 8));
                    spot_start += 2;

                    if spot_start < spot_end
                    {
                        print_string.push_str(", ");
                    }
                    else
                    {
                        break
                    }
                }

                print_string.push_str("\n");
                current_spot += 1;
            }

            print_string.push_str("\n");
        }

        std::fs::write("spots.txt", print_string).expect("unable to write file");
    }

    pub fn print_bait_ratings(&self, fix_bug: bool)
    {
        if self.version == Version::Jp
        {
            todo!("jp not implemented for bait ratings") // in jp: get offsets for 7 days
        }

        let offset = 0x1571C;

        let mut print_string = String::new();

        for bait_offset in 0x71 ..= 0xB9
        {
            for x in 0 .. DAYS - 1
            {
                let multiplier = match fix_bug
                {
                    false => 6, // bugged, in-game version
                    true  => 7, // corrected version
                };
                print_string.push_str(&format!("{}", self.get_u8(offset + bait_offset * multiplier + x)));

                if x != 5
                {
                    print_string.push_str(", ");
                }
            }
            print_string.push_str("\n");
        }

        std::fs::write("bait_ratings.txt", print_string).expect("unable to write file");
    }

    fn calculate_averages(&self)
    {
        let get_weight_as_proper_dec = |offset|
        {
            let hex_str = format!("{:04X}", self.make_u16(offset));
            match hex_str.parse::<u16>()
            {
                Ok(i) => i,
                Err(e) => panic!("failed to parse: {e}"),
            }
        };

        let mut weights = [[0; 10]; DAYS];
        for d in 0 .. DAYS
        {
            let base_offset = [0x17D2, 0x17CE];
            let offset = self.make_u16(base_offset[self.version.offset()] + d * 2) as usize;

            for x in 0 .. weights.len()
            {
                weights[d][x] = get_weight_as_proper_dec(offset + x * 2);
            }
        }

        let mut print_string = String::new();

        let days = [1, 2, 2, 3, 3, 3, 4]; //unknown days in bonus
        let names = ["spring", "summer", "fall", "winter 1", "winter 2", "championship", "bonus"];
        for x in 0 .. DAYS - 1 // todo: in jp, calc all stages
        {
            print_string.push_str(&format!("{:<12} | ", names[x]));
            print_string.push_str(&Rom::test_perm(weights[x], days[x]));
        }

        let mut filename = String::from("winning_averages");
        filename.push_str
        (
            match self.version
            {
                Version::Us => "_us.txt",
                Version::Jp => "_jp.txt",
            }
        );

        std::fs::write(filename, print_string).expect("unable to write file");
    }

    fn test_perm(weights: [u16; 10], days: usize) -> String
    {
        let mut perm = vec![0_u8; days];

        let mut list_of_best = Vec::new();

        'outer: loop
        {
            let mut sum_per_fisher = [0; 10];

            for fisher in 0 .. sum_per_fisher.len()
            {
                for day in 0 .. days
                {
                    let mut weight_rotate = weights;
                    weight_rotate.rotate_right(perm[day] as usize);

                    sum_per_fisher[fisher] += weight_rotate[fisher];
                }
            }

            let best = sum_per_fisher.iter().max().unwrap().clone();
            list_of_best.push(best);

            perm[0] += 1;
            for x in 0 .. perm.len()
            {
                if perm[x] == 8
                {
                    if x == perm.len() - 1
                    {
                        break 'outer;
                    }

                    perm[x] = 0;
                    perm[x + 1] += 1;
                }
            }
        }

        let sum = list_of_best.iter().map(|&x| x as u32).sum::<u32>();
        let variance = 0.165;
        let count = list_of_best.len() as f64;

        let avg = ((sum as f64 / 100.0) + variance * count) / count;

        format!("{avg:.2}\n")
    }
}
