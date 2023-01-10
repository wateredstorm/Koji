import { ToConvert } from '@assets/types'
import { Code } from '@components/Code'
import { usePersist } from '@hooks/usePersist'
import { Typography } from '@mui/material'
import { convert } from '@services/fetches'
import { safeParse } from '@services/utils'
import * as React from 'react'
import { useInput } from 'react-admin'

export default function CodeInput({
  source,
  label,
}: {
  source: string
  label?: string
}) {
  const { field } = useInput({ source })
  const [error, setError] = React.useState('')
  const simplifyPolygons = usePersist((s) => s.simplifyPolygons)

  return (
    <>
      <Typography variant="subtitle2">{label}</Typography>
      <Code
        width="75vw"
        maxHeight="50vh"
        code={
          typeof field.value === 'object'
            ? JSON.stringify(field.value, null, 2)
            : field.value
        }
        setCode={(newCode) => {
          field.onChange({ target: { value: newCode } })
        }}
        onBlurCapture={async () => {
          const geofence = safeParse<ToConvert>(field.value)
          if (!geofence.error) {
            await convert(geofence.value, 'feature', simplifyPolygons).then(
              (res) => {
                if (Array.isArray(res)) {
                  setError(
                    'Warning, multiple features were found, you should only assign one feature!',
                  )
                } else {
                  field.onChange({
                    target: { value: JSON.stringify(res, null, 2) },
                  })
                  setError('')
                }
              },
            )
          }
        }}
      />
      <Typography color="error">{error}</Typography>
    </>
  )
}