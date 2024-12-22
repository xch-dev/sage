export async function uploadToDexie(
  offer: string,
  testnet: boolean,
): Promise<string> {
  const response = await fetch(
    `https://${testnet ? 'api-testnet' : 'api'}.dexie.space/v1/offers`,
    {
      method: 'POST',
      body: JSON.stringify({ offer }),
      headers: {
        'Content-Type': 'application/json',
      },
    },
  );

  const data = await response.json();
  console.log(data);

  return `https://${testnet ? 'testnet.' : ''}dexie.space/offers/${data.id}`;
}

export async function uploadToMintGarden(
  offer: string,
  testnet: boolean,
): Promise<string> {
  const response = await fetch(
    `https://${testnet ? 'api.testnet' : 'api'}.mintgarden.io/offer`,
    {
      method: 'POST',
      body: JSON.stringify({ offer }),
      headers: {
        'Content-Type': 'application/json',
      },
    },
  );

  const data = await response.json();
  console.log(data);

  return `https://${testnet ? 'testnet.' : ''}mintgarden.io/offers/${data.offer.id}`;
}
