/* eslint-disable no-param-reassign */
import * as React from 'react'
import { FeatureGroup, useMap } from 'react-leaflet'
import * as L from 'leaflet'
import type { MultiPolygon, Point, Polygon } from 'geojson'
import 'leaflet-arrowheads'
import { GeomanControls } from 'react-leaflet-geoman-v2'

import type { Feature } from '@assets/types'
import { useStatic } from '@hooks/useStatic'
import { usePersist } from '@hooks/usePersist'
import { useShapes } from '@hooks/useShapes'
import {
  buildShortcutKey,
  getPointColor,
  getPolygonColor,
  reverseObject,
} from '@services/utils'

export function Drawing() {
  const snappable = usePersist((s) => s.snappable)
  const continueDrawing = usePersist((s) => s.continueDrawing)
  const radius = usePersist((s) => s.radius)
  const calculationMode = usePersist((s) => s.calculation_mode)
  const map = useMap()

  const ref = React.useRef<L.FeatureGroup>(null)

  const revertColors = () =>
    map.pm.getGeomanLayers().forEach((layer) => {
      if (layer instanceof L.Polygon) {
        layer.setStyle({
          color: getPolygonColor(`${layer.feature?.id}`),
        })
      } else if (layer instanceof L.Circle) {
        layer.setStyle({
          color: getPointColor(
            `${layer.feature?.id}`,
            layer.feature?.geometry?.type || 'MultiPoint',
            typeof layer.feature?.id === 'number' ? layer.feature.id / 10 : 0,
          ),
        })
      }
    })

  return (
    <FeatureGroup ref={ref}>
      <GeomanControls
        options={{
          position: 'topright',
          drawText: false,
          drawMarker: false,
          drawCircleMarker: false,
          drawCircle: true,
          drawRectangle: true,
          drawPolyline: false,
          drawPolygon: true,
          customControls: true,
        }}
        globalOptions={{
          continueDrawing,
          snappable,
          radiusEditCircle: false,
          templineStyle: {
            radius: calculationMode === 'Radius' ? radius || 70 : 100,
          },
        }}
        onMount={() => {
          map.pm.Toolbar.changeActionsOfControl('removalMode', [
            {
              text: 'Lines',
              onClick() {
                useShapes.getState().setters.remove('LineString')
                useShapes.getState().setters.remove('MultiLineString')
              },
            },
            {
              text: 'Circles',
              onClick() {
                useShapes.getState().setters.remove('Point')
                useShapes.getState().setters.remove('MultiPoint')
              },
            },
            {
              text: 'Polygons',
              onClick() {
                useShapes.getState().setters.remove('Polygon')
                useShapes.getState().setters.remove('MultiPolygon')
              },
            },
            {
              text: 'Finish',
              onClick() {
                map.pm.disableGlobalRemovalMode()
              },
            },
          ])
          map.pm.Toolbar.changeActionsOfControl('drawCircle', [
            {
              text: 'Finish',
              onClick() {
                map.pm.disableDraw()
                const {
                  setters: { activeRoute },
                } = useShapes.getState()

                activeRoute()
                useShapes.setState((prev) => ({
                  newPoints: [],
                  newRouteCount: prev.newRouteCount + 1,
                }))
              },
            },
            {
              text: 'New Route',
              onClick() {
                const {
                  setters: { activeRoute },
                  newRouteCount,
                } = useShapes.getState()

                activeRoute(`${newRouteCount + 1}__unset__CLIENT`)
                useShapes.setState((prev) => ({
                  newPoints: [],
                  newRouteCount: prev.newRouteCount + 1,
                }))
              },
            },
            {
              text: 'Cancel',
              onClick() {
                const {
                  setters: { remove, activeRoute },
                  newPoints,
                } = useShapes.getState()
                map.pm.disableDraw()

                newPoints.forEach((point) => {
                  remove('Point', point)
                })
                activeRoute()
                useShapes.setState((prev) => ({
                  newPoints: [],
                  newRouteCount: prev.newRouteCount + 1,
                }))
              },
            },
          ])
          if (!map.pm.Toolbar.controlExists('mergeMode')) {
            map.pm.Toolbar.createCustomControl({
              name: 'mergeMode',
              block: 'custom',
              title: 'Merge Shapes',
              className: 'leaflet-button-merge',
              toggle: true,
              actions: [
                {
                  text: 'Merge',
                  onClick() {
                    useShapes.getState().setters.combine()
                  },
                },
                {
                  text: 'Merge All',
                  onClick() {
                    useShapes.getState().setters.combine(true)
                  },
                },
                'cancel',
              ],
            })
          }
        }}
        onCreate={async ({ layer, shape }) => {
          if (ref.current && ref.current.hasLayer(layer)) {
            const id = ref.current.getLayerId(layer)
            const { setters, getters, setShapes } = useShapes.getState()
            switch (shape) {
              case 'Rectangle':
              case 'Polygon':
                if (layer instanceof L.Polygon) {
                  const feature = layer.toGeoJSON()
                  feature.id = id.toString()
                  if (feature.geometry.type === 'Polygon') {
                    setShapes('Polygon', (prev) => ({
                      ...prev,
                      [id.toString()]: feature as Feature<Polygon>,
                    }))
                  } else if (feature.geometry.type === 'MultiPolygon') {
                    setShapes('MultiPolygon', (prev) => ({
                      ...prev,
                      [id.toString()]: feature as Feature<MultiPolygon>,
                    }))
                  }
                }
                break
              case 'Circle':
                if (layer instanceof L.Circle) {
                  const feature = layer.toGeoJSON() as Feature<Point>
                  feature.id = id
                  const first = getters.getFirst()
                  const last = getters.getLast()

                  if (feature.properties) {
                    if (typeof first?.id === 'number') {
                      feature.properties.__forward = first.id
                    }
                    if (typeof last?.id === 'number') {
                      feature.properties.__backward = last.id
                    }
                    feature.properties.__multipoint_id = `${
                      useShapes.getState().newRouteCount
                    }__unset__CLIENT`
                  }
                  if (last?.properties) {
                    last.properties.__forward = id
                  }
                  if (first?.properties) {
                    first.properties.__backward = id
                  }
                  if (last && first) {
                    setShapes('LineString', (prev) => {
                      const newState: typeof prev = {
                        ...prev,
                        [`${+last.id}__${+feature.id}`]: {
                          type: 'Feature',
                          id: `${+last.id}__${+feature.id}`,
                          properties: {
                            __start: +last.id,
                            __end: +feature.id,
                          },
                          geometry: {
                            type: 'LineString',
                            coordinates: [
                              last.geometry.coordinates,
                              feature.geometry.coordinates,
                            ],
                          },
                        },
                      }
                      if (Object.keys(useShapes.getState().Point).length > 1) {
                        if (
                          typeof feature.id === 'number' &&
                          typeof first.id === 'number'
                        ) {
                          newState[`${feature.id}__${first.id}`] = {
                            type: 'Feature',
                            id: `${feature.id}__${first.id}`,
                            properties: {
                              __start: feature.id,
                              __end: first.id,
                            },
                            geometry: {
                              type: 'LineString',
                              coordinates: [
                                feature.geometry.coordinates,
                                first.geometry.coordinates,
                              ],
                            },
                          }
                        }
                      }
                      return newState
                    })
                    setters.remove('LineString', `${last.id}__${first.id}`)
                  }
                  setShapes('newPoints', (prev) => [...prev, id])
                  setShapes('Point', (prev) => ({
                    ...prev,
                    [id]: feature,
                  }))
                  if (last) {
                    setShapes('Point', (prev) => ({
                      ...prev,
                      [last?.id]: last,
                    }))
                  }
                  if (!first) {
                    setShapes('firstPoint', id)
                  } else {
                    setShapes('Point', (prev) => ({
                      ...prev,
                      [first?.id]: first,
                    }))
                  }
                  setShapes('lastPoint', id)
                }
                break
              default:
                break
            }
            ref.current.removeLayer(layer)
          }
        }}
        onGlobalDrawModeToggled={({ enabled, shape }) => {
          const { showCircles, showPolygons, setStore } = usePersist.getState()

          switch (shape) {
            case 'Circle':
              if (!showCircles && enabled) {
                setStore('showCircles', true)
              }
              useShapes.setState({ newPoints: [] })
              break
            case 'Rectangle':
            case 'Polygon':
              if (!showPolygons && enabled) {
                setStore('showPolygons', true)
              }
              break
            default:
              break
          }
          useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, drawMode: enabled }))
        }}
        onGlobalCutModeToggled={({ enabled }) =>
          useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, cutMode: enabled }))
        }
        onGlobalDragModeToggled={({ enabled }) =>
          useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, dragMode: enabled }))
        }
        onGlobalEditModeToggled={({ enabled }) => {
          return useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, editMode: enabled }))
        }}
        onGlobalRemovalModeToggled={({ enabled }) =>
          useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, removalMode: enabled }))
        }
        onGlobalRotateModeToggled={({ enabled }) =>
          useStatic
            .getState()
            .setStatic('layerEditing', (e) => ({ ...e, rotateMode: enabled }))
        }
        onButtonClick={(e) => {
          revertColors()
          if (e.btnName === 'mergeMode') {
            useShapes.setState({ combined: {} })
            useStatic.setState((prev) => ({
              combinePolyMode: !prev.combinePolyMode,
            }))
          } else {
            useStatic.setState({ combinePolyMode: false })
          }
        }}
        onKeyEvent={(e) => {
          if (e.eventType === 'keyup' || useStatic.getState().dialogs.keyboard)
            return
          const { kbShortcuts, tileServer } = usePersist.getState()
          const { tileServers } = useStatic.getState()

          const reverse = reverseObject(kbShortcuts)
          const shortcut = buildShortcutKey(e.event)
          if (reverse[shortcut]) {
            e.event.preventDefault()
            switch (reverse[shortcut]) {
              case 'drawCircle':
                if (!map.pm.globalDrawModeEnabled()) {
                  map.pm.enableDraw('Circle')
                } else {
                  map.pm.disableDraw('Circle')
                }
                break
              case 'drawRectangle':
                if (!map.pm.globalDrawModeEnabled()) {
                  map.pm.enableDraw('Rectangle')
                } else {
                  map.pm.disableDraw('Rectangle')
                }
                break
              case 'drawPolygon':
                if (!map.pm.globalDrawModeEnabled()) {
                  map.pm.enableDraw('Polygon')
                } else {
                  map.pm.disableDraw('Polygon')
                }
                break
              case 'drag':
                map.pm.toggleGlobalDragMode()
                break
              case 'edit':
                map.pm.toggleGlobalEditMode()
                break
              case 'remove':
                map.pm.toggleGlobalRemovalMode()
                break
              case 'rotate':
                map.pm.toggleGlobalRotateMode()
                break
              case 'cut':
                map.pm.toggleGlobalCutMode()
                break
              case 'setTileServer':
                {
                  const index = tileServers.findIndex(
                    (ts) => ts.url === tileServer,
                  )
                  usePersist.setState({
                    tileServer: tileServers[index + 1]
                      ? tileServers[index + 1].url
                      : tileServers[0].url,
                  })
                }
                break
              case 'theme':
                usePersist.setState({
                  darkMode: !usePersist.getState().darkMode,
                })
                break
              case 'drawer':
                usePersist.setState({
                  drawer: !usePersist.getState().drawer,
                })
                break
              case 'arrows':
                usePersist.setState({
                  showArrows: !usePersist.getState().showArrows,
                })
                break
              case 'circles':
                usePersist.setState({
                  showCircles: !usePersist.getState().showCircles,
                })
                break
              case 'lines':
                usePersist.setState({
                  showLines: !usePersist.getState().showLines,
                })
                break
              case 'polygons':
                usePersist.setState({
                  showPolygons: !usePersist.getState().showPolygons,
                })
                break
              case 'gyms':
                usePersist.setState({
                  gym: !usePersist.getState().gym,
                })
                break
              case 'pokestops':
                usePersist.setState({
                  pokestop: !usePersist.getState().pokestop,
                })
                break
              case 'spawnpoints':
                usePersist.setState({
                  spawnpoint: !usePersist.getState().spawnpoint,
                })
                break
              default:
            }
          }
        }}
        onActionClick={({ btnName, text }) => {
          if (btnName === 'mergeMode' && text === 'Cancel') {
            revertColors()
            useStatic.setState({ combinePolyMode: false })
            useShapes.setState({ combined: {} })
          }
        }}
      />
    </FeatureGroup>
  )
}

export default React.memo(Drawing)
