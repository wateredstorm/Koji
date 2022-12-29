import { usePersist } from '@hooks/usePersist'
import { getLotsOfData } from '@services/fetches'
// import { COLORS } from '@assets/constants'
import { useStatic } from '@hooks/useStatic'
import useDeepCompareEffect from 'use-deep-compare-effect'
import { useShapes } from './useShapes'

export default function useCluster(): void {
  const mode = usePersist((s) => s.mode)
  const radius = usePersist((s) => s.radius)
  const category = usePersist((s) => s.category)
  const min_points = usePersist((s) => s.min_points)
  const fast = usePersist((s) => s.fast)
  const autoMode = usePersist((s) => s.autoMode)
  const routing_time = usePersist((s) => s.routing_time)
  const only_unique = usePersist((s) => s.only_unique)
  const save_to_db = usePersist((s) => s.save_to_db)
  const last_seen = usePersist((s) => s.last_seen)
  const route_chunk_size = usePersist((s) => s.route_chunk_size)
  const drawer = usePersist((s) => s.drawer)
  const menuItem = usePersist((s) => s.menuItem)

  const geojson = useStatic((s) => s.geojson)
  const layerEditing = useStatic((s) => s.layerEditing)
  const forceFetch = useStatic((s) => s.forceFetch)
  const setStatic = useStatic((s) => s.setStatic)

  const add = useShapes((s) => s.setters.add)

  useDeepCompareEffect(() => {
    if (geojson.features.length && drawer && menuItem === 'Clustering') {
      if (Object.values(layerEditing).every((v) => !v)) {
        setStatic(
          'loading',
          Object.fromEntries(
            geojson.features
              .filter(
                (feat) =>
                  feat.geometry.type === 'Polygon' ||
                  feat.geometry.type === 'MultiPolygon',
              )
              .map((k) => [
                k.properties?.name ||
                  `${k.geometry.type}${k.id ? `-${k.id}` : ''}`,
                null,
              ]),
          ),
        )
        setStatic('totalLoadingTime', 0)
        getLotsOfData(
          mode === 'bootstrap'
            ? '/api/v1/calc/bootstrap'
            : `/api/v1/calc/${mode}/${category}`,
          setStatic,
          {
            category,
            radius,
            geojson,
            min_points,
            fast,
            routing_time,
            only_unique,
            save_to_db,
            last_seen,
            route_chunk_size,
          },
        ).then((newCollection) => {
          add(newCollection.features)
        })
      }
    }
  }, [
    autoMode
      ? {
          mode,
          radius,
          fast,
          min_points,
          category,
          geojson,
          routing_time,
          only_unique,
          save_to_db,
          last_seen,
        }
      : { forceFetch },
  ])
}
