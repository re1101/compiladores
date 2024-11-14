import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class analisisLexico {
    // Expresiones regulares
    private static final String IDENTIFIER_REGEX = "[a-zA-Z_][a-zA-Z0-9_]{0,30}";
    private static final String NUMBER_REGEX = "[0-9]+";
    private static final String ASSIGNMENT_REGEX = "=";
    private static final String LEFT_PAREN_REGEX = "\\(";
    private static final String RIGHT_PAREN_REGEX = "\\)";
    private static final String LEFT_BRACE_REGEX = "\\{";
    private static final String RIGHT_BRACE_REGEX = "\\}";
    private static final String LEFT_BRACKET_REGEX = "\\[";
    private static final String RIGHT_BRACKET_REGEX = "\\]";
    private static final String COMA_REGEX = ",";
    private static final String DOT_REGEX = ".";
    private static final String SEMICOLON_REGEX = ";";
    private static final String STRING_REGEX = "\"(([^\"\\\\]|\\\\.)*)\""; // Soporte para cadenas
    private static final String CHAR_REGEX = "'([^'\\\\]|\\\\.){1}'"; // Soporte para un solo caracter
    private static final String SINGLE_LINE_COMMENT_REGEX = "\\/\\/.*"; // Soporte para comentarios unilinea
    private static final String MULTI_LINE_COMMENT_REGEX = "/\\*([^*]|\\*(?!/))*\\*/"; //Soporte para comentarios multilinea
    private static final String COMBINED_ASSIGNMENT_REGEX = "(\\+=|-=|\\*=|/=|\\+\\+|--)";
    private static final String OPERATOR_REGEX = "[+\\-*/%^!]+";
    private static final String LOGIC_OPERATOR_REGEX = "(&&|\\|\\|)";
    private static final String BIT_OPERATOR_REGEX = "(&|\\||\\^|~|<<|>>)";
    private static final String COMPARATOR_REGEX = "(==|!=|<|<=|>|>=|<>)"; // Ajustado para comparadores
    private static final String RESERVED_WORDS_REGEX = "(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)";

    public static void main(String[] args) {
        // Ruta del archivo .txt
        String filePath = "prueba.txt";

        try (BufferedReader br = new BufferedReader(new FileReader(filePath))) {
            StringBuilder contentBuilder = new StringBuilder();
            String line;

            while ((line = br.readLine()) != null) {
                line = handleComments(line);
                contentBuilder.append(line).append(" "); // Unir las líneas en una sola
            }

            String content = contentBuilder.toString();
            String[] logicalLines = content.split("(?<=;)"); // Divide cada línea lógica por ';'

            for (String logicalLine : logicalLines) {

                logicalLine = logicalLine.trim(); // Quitar espacios
                if (!logicalLine.isEmpty()) {
                    
                    String primeLine = logicalLine;
                    
                    String auxLine = handleComments(logicalLine);
                    logicalLine = auxLine;

                    // Aplica la función que maneja cadenas y caracteres
                    auxLine = handleStringsAndChars(logicalLine);
                    logicalLine = auxLine;

                    // Paso 2: Tokeniza la línea normalmente
                    String[] tokens = logicalLine.split("(?<=\\W)|(?=\\W)"); // Separar por símbolos no alfanuméricos
                    
                    System.out.println("");
                    System.out.println("Analizando línea: " + primeLine);
                    boolean esValido = true;
                    System.out.print(">>");
                    
                    for (String token : tokens) {
                        token = token.trim();
                        if (token.isEmpty()) continue; // Ignorar tokens vacíos

                        if (isReservedWord(token)) {
                            System.out.print("|PR|");
                        } else if (isString(token)) {
                            System.out.print("|STR|");
                        } else if (isChar(token)) {
                            System.out.print("|CHR|"); // Token reconocido como carácter
                        } else if (isIdentifier(token)) {
                            System.out.print("|ID|");
                        } else if (isCombinedAssignment(token)) {
                            System.out.print("|AC|");
                        } else if (isAssignment(token)) {
                            System.out.print("|AS|");
                        } else if (isDigit(token)) {
                            System.out.print("|NU|");
                        } else if (isOperator(token)) {
                            System.out.print("|OP|");
                        } else if (isLogicOperator(token)) {
                            System.out.print("|LO|");
                        } else if (isBitOperator(token)) {
                            System.out.print("|BO|");
                        } else if (isComparator(token)) {
                            System.out.print("|CO|");
                        } else if (isLeftParen(token)) {
                            System.out.print("|LP|");
                        } else if (isRightParen(token)) {
                            System.out.print("|RP|");
                        } else if (isLeftBrace(token)) {
                            System.out.print("|LB|");
                        } else if (isRightBrace(token)) {
                            System.out.print("|RB|");
                        } else if (isLeftBracket(token)) {
                            System.out.print("|LBR|");
                        } else if (isRightBracket(token)) {
                            System.out.print("|RBR|");
                        } else if (isComa(token)) {
                            System.out.print("|COMA|");
                        } else if (isSemicolon(token)) {
                            System.out.print("|SC|");
                        } else if (isDot(token)) {
                            System.out.print("|DOT|");
                        } else {
                            System.out.println("|ERROR| Token no reconocido: " + token);
                            esValido = false;
                        }
                    }

                    if (esValido) {
                        System.out.println(" La línea es válida en C.");
                    } else {
                        System.out.println(" La línea NO es válida en C.");
                    }
                }
            }
        } catch (IOException e) {
            e.getMessage();
            e.printStackTrace();
        }
    }

    // Paso 1: Reemplaza las cadenas y caracteres por marcadores
    public static String handleStringsAndChars(String line) {

        String newLine = line;
        Pattern stringPattern = Pattern.compile(STRING_REGEX);
        Pattern charPattern = Pattern.compile(CHAR_REGEX);

        // Reemplazar cadenas por "STR"
        Matcher stringMatcher = stringPattern.matcher(newLine);
        newLine = stringMatcher.replaceAll(" STR ");

        // Reemplazar caracteres por "CHR"
        Matcher charMatcher = charPattern.matcher(newLine);
        newLine = charMatcher.replaceAll(" CHR ");

        return newLine;
    }

    // Función para manejar comentarios
    public static String handleComments(String line) {

        String newLine = line;
        Pattern singleLinePattern = Pattern.compile(SINGLE_LINE_COMMENT_REGEX);
        Pattern multiLinePattern = Pattern.compile(MULTI_LINE_COMMENT_REGEX);

        // Reemplazar comentarios de una línea por "COMMENT"
        Matcher singleLineMatcher = singleLinePattern.matcher(newLine);
        newLine = singleLineMatcher.replaceAll("");

        // Reemplazar comentarios de varias líneas por "COMMENT"
        Matcher multiLineMatcher = multiLinePattern.matcher(newLine);
        line = multiLineMatcher.replaceAll("");

        return line;
    }

    // Funciones para reconocer diferentes tipos de tokens

    public static boolean isIdentifier(String input) {
        return Pattern.matches(IDENTIFIER_REGEX, input);
    }

    public static boolean isDigit(String input) {
        return Pattern.matches(NUMBER_REGEX, input);
    }

    public static boolean isAssignment(String input) {
        return Pattern.matches(ASSIGNMENT_REGEX, input);
    }

    public static boolean isCombinedAssignment(String input) {
        return Pattern.matches(COMBINED_ASSIGNMENT_REGEX, input);
    }

    public static boolean isOperator(String input) {
        return Pattern.matches(OPERATOR_REGEX, input);
    }

    public static boolean isLogicOperator(String input) {
        return Pattern.matches(LOGIC_OPERATOR_REGEX, input);
    }

    public static boolean isBitOperator(String input) {
        return Pattern.matches(BIT_OPERATOR_REGEX, input);
    }

    public static boolean isComparator(String input) {
        return Pattern.matches(COMPARATOR_REGEX, input);
    }

    public static boolean isReservedWord(String input) {
        return Pattern.matches(RESERVED_WORDS_REGEX, input);
    }

    public static boolean isString(String input) {
        if(input.equals("STR"))
            return true;
        else
        return false; // Reconoce cadenas
    }

    public static boolean isChar(String input) {
        if(input.equals("CHR"))
            return true;
        else
        return false; // Reconoce caracteres
    }

    public static boolean isLeftParen(String input) {
        return Pattern.matches(LEFT_PAREN_REGEX, input);
    }

    public static boolean isRightParen(String input) {
        return Pattern.matches(RIGHT_PAREN_REGEX, input);
    }

    public static boolean isLeftBrace(String input) {
        return Pattern.matches(LEFT_BRACE_REGEX, input);
    }

    public static boolean isRightBrace(String input) {
        return Pattern.matches(RIGHT_BRACE_REGEX, input);
    }

    public static boolean isLeftBracket(String input) {
        return Pattern.matches(LEFT_BRACKET_REGEX, input);
    }

    public static boolean isRightBracket(String input) {
        return Pattern.matches(RIGHT_BRACKET_REGEX, input);
    }

    public static boolean isComa(String input) {
        return Pattern.matches(COMA_REGEX, input);
    }

    public static boolean isDot(String input) {
        return Pattern.matches(DOT_REGEX, input);
    }

    public static boolean isSemicolon(String input) {
        return Pattern.matches(SEMICOLON_REGEX, input);
    }
}
