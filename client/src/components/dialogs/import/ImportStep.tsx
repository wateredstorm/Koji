import * as React from 'react'
import Grid2 from '@mui/material/Unstable_Grid2/Grid2'
import Typography from '@mui/material/Typography'

import JsonFile from '@components/drawer/manage/Json'
import ShapeFile from '@components/drawer/manage/ShapeFile'
import { Divider } from '@mui/material'
import InstanceSelect from '@components/drawer/inputs/Instance'
import { useStatic } from '@hooks/useStatic'
import { RDM_FENCES, UNOWN_FENCES } from '@assets/constants'
import type { KojiKey, FeatureCollection } from '@assets/types'

import Nominatim from './Nominatim'

const ImportStep = React.forwardRef<
  HTMLDivElement,
  {
    geojson: FeatureCollection
    handleChange: (
      geojson: FeatureCollection,
      key?: string,
      deleted?: string[],
    ) => void
  }
>(({ geojson, handleChange }, ref) => {
  const scannerType = useStatic((s) => s.scannerType)

  return (
    <Grid2 container ref={ref}>
      {/* JSON */}
      <Grid2 xs={2}>
        <Typography variant="h5">JSON</Typography>
      </Grid2>
      <Grid2 xs={6}>
        <Typography sx={{ my: 1 }}>
          Upload a JSON file, such as an <code>areas.json</code> from ReactMap,
          a <code>geofence.json</code> from Poracle, or any GeoJSON.
        </Typography>
      </Grid2>
      <Grid2 xs={4}>
        <JsonFile setter={handleChange} />
      </Grid2>
      <Divider sx={{ width: '95%', my: 1 }} />
      {/* Shapefile */}
      <Grid2 xs={2}>
        <Typography variant="h5">Shapefile</Typography>
      </Grid2>
      <Grid2 xs={6}>
        <Typography sx={{ my: 1 }}>
          Accepts either a <code>.shp</code> file or a combination of{' '}
          <code>.shp</code> and <code>.dbf</code> files to add extra
          information.
        </Typography>
      </Grid2>
      <Grid2 xs={4}>
        <ShapeFile setter={handleChange} />
      </Grid2>
      <Divider sx={{ width: '95%', my: 1 }} />
      {/* Scanner */}
      <Grid2 xs={2}>
        <Typography variant="h5">Scanner</Typography>
      </Grid2>
      <Grid2 xs={6}>
        <Typography sx={{ my: 1 }}>
          Import fences directly from your scanner database.
        </Typography>
      </Grid2>
      <Grid2 xs={4}>
        <InstanceSelect
          setGeojson={(incoming, deleted) => {
            handleChange(
              {
                ...incoming,
                features: incoming.features
                  .filter(
                    (feat) => !geojson.features.find((x) => feat.id === x.id),
                  )
                  .map((feat) => ({
                    ...feat,
                    id: `${feat.properties?.__id}__${feat.properties?.__mode}__SCANNER`,
                    properties: {
                      ...feat.properties,
                      name: feat.properties?.__name,
                      mode: feat.properties?.__mode,
                      __scanner: true,
                    },
                  })),
              },
              '__scanner',
              deleted,
            )
          }}
          controlled
          filters={scannerType === 'rdm' ? RDM_FENCES : UNOWN_FENCES}
          initialState={geojson.features
            .filter((feat) => feat.properties?.__scanner)
            .map((feat) => feat.id as KojiKey)}
        />
      </Grid2>
      <Divider sx={{ width: '95%', my: 1 }} />
      {/* Nominatim */}
      <Grid2 xs={2}>
        <Typography variant="h5">Nominatim</Typography>
      </Grid2>
      <Grid2 xs={10}>
        <Nominatim
          features={geojson.features.filter(
            (feat) => feat.properties?.__nominatim,
          )}
          handleChange={handleChange}
        />
      </Grid2>
    </Grid2>
  )
})

ImportStep.displayName = 'ImportStep'

export default ImportStep
