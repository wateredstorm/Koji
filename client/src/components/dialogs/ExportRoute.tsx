/* eslint-disable react/no-array-index-key */
import * as React from 'react'
import {
  Dialog,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  List,
  ListItemText,
  ListSubheader,
  IconButton,
  Box,
  CircularProgress,
  Typography,
} from '@mui/material'
import Grid2 from '@mui/material/Unstable_Grid2/Grid2'
import ContentCopy from '@mui/icons-material/ContentCopy'
import type { Feature, FeatureCollection } from '@assets/types'
import useDeepCompareEffect from 'use-deep-compare-effect'
import distance from '@turf/distance'

import { useStatic } from '@hooks/useStatic'
import { convert } from '@services/fetches'

import DialogHeader from './Header'

interface Props {
  open: string
  setOpen: (open: string) => void
  geojson: FeatureCollection
}

export default function ExportRoute({ open, setOpen, geojson }: Props) {
  const scannerType = useStatic((s) => s.scannerType)

  const [loading, setLoading] = React.useState(false)
  const [route, setRoute] = React.useState<number[][][]>([])
  const [stats, setStats] = React.useState<{
    max: number
    total: number
    count: number
  }>({ max: 0, total: 0, count: 0 })

  const getRoutes = async (doNotSet = false) => {
    const points = geojson.features.filter((f) => f.geometry?.type === 'Point')
    const mergedPoints = points.length
      ? await convert<Feature[]>(
          points,
          'featureVec',
          false,
          '/api/v1/convert/merge_points',
        )
      : []
    const newGeojson = {
      ...geojson,
      features: [
        ...mergedPoints,
        ...geojson.features.filter((f) => f.geometry?.type !== 'Point'),
      ],
    }
    if (doNotSet) {
      return convert<string>(
        newGeojson,
        scannerType === 'rdm' ? 'text' : 'altText',
        false,
      )
    }
    const newCode = await convert<number[][][]>(newGeojson, 'multiArray', false)

    let max = 0
    let total = 0
    let count = 0
    const newRoute = newCode.map((eachRoute) => {
      return eachRoute.map((point, j) => {
        const next = j ? eachRoute[j + 1] : eachRoute.at(-1)
        if (next) {
          const dis = distance(point, next, { units: 'meters' })
          if (dis > max) max = dis
          total += dis
        }
        count++
        return [+point[0].toFixed(6), +point[1].toFixed(6)]
      })
    })
    setStats({
      max,
      total,
      count,
    })
    setRoute(newRoute)
    return ''
  }

  const getClipboard = async (inputRoute?: number[][]) => {
    if (typeof ClipboardItem && navigator.clipboard.write) {
      // Safari bs (also works for Chrome)
      const text = new ClipboardItem({
        'text/plain': (inputRoute
          ? convert<string>(
              inputRoute,
              scannerType === 'rdm' ? 'text' : 'altText',
              false,
            )
          : getRoutes(true)
        ).then(
          (input) =>
            new Blob([input], {
              type: 'text/plain',
            }),
        ),
      })
      navigator.clipboard.write([text])
    } else {
      // Firefox
      navigator.clipboard.writeText(
        await (inputRoute
          ? convert<string>(
              inputRoute,
              scannerType === 'rdm' ? 'text' : 'altText',
              false,
            )
          : getRoutes(true)),
      )
    }
  }

  useDeepCompareEffect(() => {
    if (open === 'route') {
      setLoading(true)
      getRoutes().then(() => setLoading(false))
    }
  }, [geojson, open])

  return (
    <Dialog open={open === 'route'} maxWidth="xl" onClose={() => setOpen('')}>
      <DialogHeader action={() => setOpen('')}>Export Route</DialogHeader>
      <DialogContent>
        {loading ? (
          <Box sx={{ display: 'flex', height: '50vh', width: '40vw' }}>
            <CircularProgress />
            <Typography variant="caption">Loading</Typography>
          </Box>
        ) : (
          <Grid2 container>
            <Grid2
              container
              xs={7}
              height="50vh"
              overflow="auto"
              border="2px grey solid"
              borderRadius={2}
              mx={2}
              alignItems="center"
              justifyContent="center"
            >
              <List sx={{ width: '90%', mx: 'auto' }}>
                {route.map((feat, i) => {
                  return (
                    <React.Fragment key={i}>
                      <ListSubheader>
                        <Grid2 container justifyContent="space-around">
                          <Grid2 xs={3}>
                            <IconButton
                              onPointerDown={() => getClipboard(feat)}
                            >
                              <ContentCopy />
                            </IconButton>
                          </Grid2>
                          <Grid2 xs={9}>[Geofence {i + 1}]</Grid2>
                        </Grid2>
                      </ListSubheader>
                      {feat.map((point, j) => (
                        <ListItemText
                          key={`${i}-${j}-${point.join('')}`}
                          primary={`${point[0]}, ${point[1]}`}
                          primaryTypographyProps={{ variant: 'caption' }}
                          sx={{ w: '100%', mx: 'auto' }}
                        />
                      ))}
                    </React.Fragment>
                  )
                })}
              </List>
            </Grid2>
            <Grid2
              container
              xs={4}
              direction="column"
              alignItems="center"
              justifyContent="space-around"
              height="50vh"
            >
              <Grid2>
                <TextField
                  value={stats.count}
                  label="Count"
                  fullWidth
                  disabled
                />
              </Grid2>
              <Grid2>
                <TextField
                  value={stats.max?.toFixed(2) || 0}
                  label="Max"
                  fullWidth
                  InputProps={{ endAdornment: 'm' }}
                  disabled
                />
              </Grid2>
              <Grid2>
                <TextField
                  value={(stats.total / (stats.count || 1))?.toFixed(2) || 0}
                  label="Average"
                  fullWidth
                  InputProps={{ endAdornment: 'm' }}
                  disabled
                />
              </Grid2>
              <Grid2>
                <TextField
                  value={stats.total?.toFixed(2) || 0}
                  label="Total"
                  fullWidth
                  InputProps={{ endAdornment: 'm' }}
                  disabled
                />
              </Grid2>
            </Grid2>
          </Grid2>
        )}
      </DialogContent>
      <DialogActions>
        <Button onPointerDown={() => getClipboard()}>Copy to Clipboard</Button>
        <Button onClick={() => setOpen('')}>Close</Button>
      </DialogActions>
    </Dialog>
  )
}
