// The roster order is also the visual and keyboard order in character selection.
export const CHARACTERS = [
  {
    id: 'dante',
    name: 'DANTE',
    number: '01',
    gender: 'HOMEM',
    role: 'Vanguarda',
    color: '#ffbb79',
    signature: 'O PRIMEIRO A ENTRAR.',
    bio: 'Ex-resgatista dos túneis do Distrito Chuva. Conhece cada rota de emergência e nunca deixa uma voz sem resposta.',
    intro: 'Dante, especialista em resgate e a última pessoa da sua unidade fora da rede',
  },
  {
    id: 'kaia',
    name: 'KAIA',
    number: '09',
    gender: 'MULHER',
    role: 'Reconhecimento',
    color: '#6ef5d0',
    signature: 'UM PASSO À FRENTE DO SINAL.',
    bio: 'A agente que encontrou a frequência de Lira. Silenciosa nos telhados, incansável quando a cidade precisa dela.',
    intro: 'Kaia, especialista em reconhecimento e a última pessoa da sua unidade fora da rede',
  },
  {
    id: 'ravi',
    name: 'RAVI',
    number: '42',
    gender: 'HOMEM',
    role: 'Infiltração',
    color: '#a3b8ff',
    signature: 'TODA REDE TEM UMA BRECHA.',
    bio: 'Um antigo técnico da HELIX que conhece os segredos da usina. Hoje, usa esse conhecimento para devolver a cidade às pessoas.',
    intro: 'Ravi, especialista em infiltração e a última pessoa da sua unidade fora da rede',
  },
  {
    id: 'nika',
    name: 'NIKA',
    number: '07',
    gender: 'MULHER',
    role: 'Operações especiais',
    color: '#ee9ba9',
    signature: 'NINGUÉM FICA PARA TRÁS.',
    bio: 'Mensageira da resistência, atravessou a cidade durante o apagão. Carrega os nomes de quem ainda espera pelo amanhecer.',
    intro:
      'Nika, especialista em operações especiais e a última pessoa da sua unidade fora da rede',
  },
];
export const getCharacter = (id) =>
  CHARACTERS.find((character) => character.id === id) || CHARACTERS[1];
export function selectCharacter(profile, id) {
  if (!CHARACTERS.some((character) => character.id === id)) return false;
  profile.character = id;
  return true;
}
