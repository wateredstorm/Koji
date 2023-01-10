import * as React from 'react'
import {
  AutocompleteArrayInput,
  Edit,
  ReferenceArrayInput,
  SimpleForm,
  TextInput,
  useRecordContext,
} from 'react-admin'

import { ClientProject, KojiGeofence } from '@assets/types'

const transformPayload = async (project: ClientProject) => {
  if (Array.isArray(project.related)) {
    await fetch(`/internal/admin/geofence_project/project/${project.id}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(project.related),
    })
  }
  return project
}

function OptionRenderer() {
  const record = useRecordContext()
  return <span>{record.name}</span>
}
const inputText = (choice: KojiGeofence) => choice.name

const matchSuggestion = (filter: string, choice: KojiGeofence) => {
  return choice.name.toLowerCase().includes(filter.toLowerCase())
}

export default function ProjectEdit() {
  return (
    <Edit mutationMode="pessimistic" transform={transformPayload}>
      <SimpleForm>
        <TextInput source="name" fullWidth isRequired />
        <ReferenceArrayInput
          source="related"
          reference="geofence"
          label="Geofences"
          perPage={1000}
          sort={{ field: 'name', order: 'ASC' }}
          alwaysOn={false}
        >
          <AutocompleteArrayInput
            optionText={<OptionRenderer />}
            inputText={inputText}
            matchSuggestion={matchSuggestion}
            disableCloseOnSelect
            label="Related Geofences"
            fullWidth
            blurOnSelect={false}
          />
        </ReferenceArrayInput>
      </SimpleForm>
    </Edit>
  )
}
