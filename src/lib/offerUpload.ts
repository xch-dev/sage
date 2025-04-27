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

  return dexieLink(data.id, testnet);
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

  return mintGardenLink(data.offer.id, testnet);
}

export function dexieLink(offerId: string, testnet: boolean) {
  return `https://${testnet ? 'testnet.' : ''}dexie.space/offers/${offerId}`;
}

export function mintGardenLink(offerId: string, testnet: boolean) {
  return `https://${testnet ? 'testnet.' : ''}mintgarden.io/offers/${offerId}`;
}

export async function offerIsOnDexie(
  offerId: string,
  isTestnet: boolean,
): Promise<boolean> {
  try {
    const response = await fetch(
      `https://${isTestnet ? 'testnet.' : ''}api.dexie.space/v1/offers/${offerId}`,
    );
    const data = await response.json();
    return data.success === true;
  } catch {
    return false;
  }
}
