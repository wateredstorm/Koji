import * as React from 'react'
import { Create, SimpleForm, useNotify, useRedirect } from 'react-admin'
import { Divider, Typography } from '@mui/material'
import { AdminGeofence } from '@assets/types'

import GeofenceCreateButton from './CreateDialog'
import GeofenceForm from './GeofenceForm'

const transformPayload = async (geofence: AdminGeofence) => {
  return {
    ...geofence,
    id: 0,
    geometry: {
      ...JSON.parse(geofence.geometry.toString()),
    },
  }
}

export default function GeofenceCreate() {
  const notify = useNotify()
  const redirect = useRedirect()

  const onSuccess = () => {
    notify('Geofence created successfully')
    redirect('list', 'geofence')
  }

  return (
    <Create
      title="Create a Geofence"
      mutationOptions={{ onSuccess }}
      transform={transformPayload}
    >
      <SimpleForm>
        <Typography>Create Multiple</Typography>
        <GeofenceCreateButton>Open the Wizard</GeofenceCreateButton>
        <Divider sx={{ my: 2 }} />
        <Typography>Create One</Typography>
        <GeofenceForm />
      </SimpleForm>
    </Create>
  )
}
