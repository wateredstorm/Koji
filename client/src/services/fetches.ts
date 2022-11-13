import type { Map } from 'leaflet'

import type { CombinedState, Data } from '@assets/types'
import type { UseStore } from '@hooks/useStore'
import type { UseStatic } from '@hooks/useStatic'

import { getMapBounds, convertGeojson } from './utils'

export async function getData<T>(
  url: string,
  settings: CombinedState & { area?: [number, number][] } = {},
): Promise<T | null> {
  try {
    const data = Object.keys(settings).length
      ? await fetch(url, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(settings),
        })
      : await fetch(url)
    const body = await data.json()
    if (!data.ok) {
      throw new Error(body.message)
    }
    return body
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error(e)
    return null
    // return { error: e instanceof Error ? e.message : 'Unknown Error' }
  }
}

export async function getLotsOfData(
  url: string,
  settings: CombinedState = {},
): Promise<[number, number][][]> {
  const flat = convertGeojson(settings.geojson)
  const results = await Promise.all(
    flat.map((area) =>
      fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          ...settings,
          return_type: 'multi_array',
          devices: Math.max(
            Math.floor((settings.devices || 1) / flat.length),
            1,
          ),
          area,
        }),
      }).then((res) => res.json()),
    ),
  )
  return results.flatMap((r) => r)
}

export async function getMarkers(
  map: Map,
  data: UseStore['data'],
  geojson: UseStatic['geojson'],
  enableStops: boolean,
  enableSpawnpoints: boolean,
  enableGyms: boolean,
): Promise<Data> {
  const [pokestops, gyms, spawnpoints] = await Promise.all(
    [
      enableStops ? 'pokestop' : '',
      enableGyms ? 'gym' : '',
      enableSpawnpoints ? 'spawnpoint' : '',
    ].map(async (category) =>
      category && (data === 'area' ? geojson.features.length : true)
        ? fetch(
            `/api/data/${data}/${category}`,
            data === 'all'
              ? undefined
              : {
                  method: 'POST',
                  headers: {
                    'Content-Type': 'application/json',
                  },
                  body: JSON.stringify(
                    data === 'bound'
                      ? getMapBounds(map)
                      : { area: convertGeojson(geojson) },
                  ),
                },
          ).then((res) => res.json())
        : [],
    ),
  )
  return {
    pokestops: Array.isArray(pokestops) ? pokestops : [],
    gyms: Array.isArray(gyms) ? gyms : [],
    spawnpoints: Array.isArray(spawnpoints) ? spawnpoints : [],
  }
}
