import type { Data } from '@assets/types'
import type { UseStore } from '@hooks/useStore'

export async function getData<T>(
  url: string,
  instance?: UseStore['apiSettings'],
): Promise<T> {
  try {
    const data = instance
      ? await fetch(url, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            instance: instance.instance || '',
            radius: instance.radius || 0.0,
            generations: instance.generations || 0,
            mode: instance.mode || '',
          }),
        })
      : await fetch(url)
    if (!data.ok) {
      throw new Error('Failed to fetch data')
    }
    return await data.json()
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error(e)
    return {} as T
  }
}

export async function getMarkers(): Promise<Data> {
  const [pokestops, gyms, spawnpoints] = await Promise.all([
    fetch('/api/pokestop/all').then((res) => res.json()),
    fetch('/api/gym/all').then((res) => res.json()),
    fetch('/api/spawnpoint/all').then((res) => res.json()),
  ])
  return { pokestops, gyms, spawnpoints }
}
