import { Box, Typography } from '@mui/material';

export default function ErrorPage({
  message,
  error,
  redirectURL,
}: {
  message: string;
  error: any;
  redirectURL: string;
}) {
  setTimeout(() => window.location.replace(redirectURL), 500000);
  console.log(error);

  return (
    <Box
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <Typography variant="h4">{message}</Typography>
      <Typography variant="h5">{error.message}</Typography>
    </Box>
  );
}
