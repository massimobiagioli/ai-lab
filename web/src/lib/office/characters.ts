/** Character roster from Appendix C.1 (exactly 10). */
export type CharacterId =
  | 'veteran_lead'
  | 'systems_analyst'
  | 'scrappy_dev'
  | 'staff_architect'
  | 'precise_consultant'
  | 'confident_muse'
  | 'steady_operator'
  | 'eager_intern'
  | 'pixel_artisan'
  | 'quiet_professor';

export type HairStyle =
  | 'silver'
  | 'curly'
  | 'buzz'
  | 'tall-curly'
  | 'neat'
  | 'long'
  | 'auburn'
  | 'cap'
  | 'beanie'
  | 'gray';

export type Accessory =
  | 'glasses'
  | 'notebook'
  | 'headphones'
  | 'tablet'
  | 'tie'
  | 'headset'
  | 'lanyard'
  | 'stylus'
  | 'beard'
  | 'mug';

export interface CharacterProfile {
  readonly id: CharacterId;
  readonly name: string;
  readonly skin: string;
  readonly skinShade: string;
  readonly hair: string;
  readonly hairStyle: HairStyle;
  readonly outfit: string;
  readonly outfitAccent: string;
  readonly accessory: Accessory;
  readonly fit: readonly string[];
  readonly speech: string;
}

export const CHARACTERS: readonly CharacterProfile[] = [
  {
    id: 'veteran_lead',
    name: 'The Veteran Lead',
    skin: '#E8B98F',
    skinShade: '#C98F66',
    hair: '#C9C9C9',
    hairStyle: 'silver',
    outfit: '#6B4F3A',
    outfitAccent: '#F2B84B',
    accessory: 'glasses',
    fit: ['tech_lead'],
    speech: 'Measured, decisive, one sharp question at a time.',
  },
  {
    id: 'systems_analyst',
    name: 'The Systems Analyst',
    skin: '#8D5524',
    skinShade: '#6E3F18',
    hair: '#2B1D16',
    hairStyle: 'curly',
    outfit: '#2E3A6B',
    outfitAccent: '#57C785',
    accessory: 'notebook',
    fit: ['analyst'],
    speech: 'Precise, list-driven, cites sources.',
  },
  {
    id: 'scrappy_dev',
    name: 'The Scrappy Dev',
    skin: '#F1C27D',
    skinShade: '#CE9B55',
    hair: '#1B1B1B',
    hairStyle: 'buzz',
    outfit: '#C0392B',
    outfitAccent: '#F2E9D8',
    accessory: 'headphones',
    fit: ['dev'],
    speech: 'Fast, informal, explains with tiny examples.',
  },
  {
    id: 'staff_architect',
    name: 'The Staff Architect',
    skin: '#5C3317',
    skinShade: '#42230D',
    hair: '#141414',
    hairStyle: 'tall-curly',
    outfit: '#141B3C',
    outfitAccent: '#9FB3E6',
    accessory: 'tablet',
    fit: ['dev', 'tech_lead'],
    speech: 'Big-picture, diagrams first, names trade-offs.',
  },
  {
    id: 'precise_consultant',
    name: 'The Precise Consultant',
    skin: '#C68642',
    skinShade: '#A5682B',
    hair: '#1A1A1A',
    hairStyle: 'neat',
    outfit: '#2E3A6B',
    outfitAccent: '#F2E9D8',
    accessory: 'tie',
    fit: ['analyst', 'qa'],
    speech: 'Formal, exact, quotes requirement IDs.',
  },
  {
    id: 'confident_muse',
    name: 'The Confident Muse',
    skin: '#F1C27D',
    skinShade: '#CE9B55',
    hair: '#3B1F14',
    hairStyle: 'long',
    outfit: '#C0392B',
    outfitAccent: '#C45AA0',
    accessory: 'tablet',
    fit: ['analyst', 'tech_lead'],
    speech: 'Bold, persuasive, outcome-oriented.',
  },
  {
    id: 'steady_operator',
    name: 'The Steady Operator',
    skin: '#E8B98F',
    skinShade: '#C98F66',
    hair: '#7A5230',
    hairStyle: 'auburn',
    outfit: '#2A9D8F',
    outfitAccent: '#F2B84B',
    accessory: 'headset',
    fit: ['qa', 'tech_lead'],
    speech: 'Warm, pragmatic, keeps people unblocked.',
  },
  {
    id: 'eager_intern',
    name: 'The Eager Intern',
    skin: '#F1C27D',
    skinShade: '#CE9B55',
    hair: '#3B2A1A',
    hairStyle: 'cap',
    outfit: '#57C785',
    outfitAccent: '#F2B84B',
    accessory: 'lanyard',
    fit: ['dev', 'qa'],
    speech: 'Enthusiastic, asks why, volunteers.',
  },
  {
    id: 'pixel_artisan',
    name: 'The Pixel Artisan',
    skin: '#C68642',
    skinShade: '#A5682B',
    hair: '#2B1D16',
    hairStyle: 'beanie',
    outfit: '#E07A3F',
    outfitAccent: '#2A9D8F',
    accessory: 'stylus',
    fit: ['qa', 'dev'],
    speech: 'Detail-obsessed, talks in frames and pixels.',
  },
  {
    id: 'quiet_professor',
    name: 'The Quiet Professor',
    skin: '#E8B98F',
    skinShade: '#C98F66',
    hair: '#B0B0B0',
    hairStyle: 'gray',
    outfit: '#6B4F3A',
    outfitAccent: '#9FB3E6',
    accessory: 'beard',
    fit: ['qa', 'analyst'],
    speech: 'Reflective, principle-driven, spots edge cases.',
  },
];

export const CHARACTER_BY_ID: Readonly<Record<string, CharacterProfile>> = Object.fromEntries(
  CHARACTERS.map((profile) => [profile.id, profile]),
);

export function characterOrDefault(id: string | undefined): CharacterProfile {
  return CHARACTER_BY_ID[id ?? ''] ?? CHARACTERS[0]!;
}
